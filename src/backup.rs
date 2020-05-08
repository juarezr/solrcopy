use crossbeam_channel::{bounded, Receiver, Sender};
use crossbeam_utils::thread;
use log::{debug, error, info, trace};

use std::sync::{atomic::AtomicBool, Arc};
use std::{path::PathBuf, time::Instant};

use crate::{
    args::Backup,
    bars::foreach_progress,
    connection::SolrClient,
    fails::*,
    helpers::*,
    save::Archiver,
    state::*,
    steps::{Documents, Requests, SolrCore, Step},
};

pub(crate) fn backup_main(params: Backup) -> BoxedError {
    debug!("  {:?}", params);

    let core_info = params.inspect_core()?;

    let end_limit = params.get_docs_to_retrieve(&core_info);
    let num_retrieving = end_limit - params.skip;
    let num_found = core_info.num_found.to_u64();
    let must_match = if params.workaround_shards > 0 { num_found } else { 0 };
    let mut retrieved = 0;

    info!(
        "Starting retrieving between {} and {} from {} documents of solr core {}.",
        params.skip + 1,
        end_limit,
        num_found,
        params.options.core
    );

    let ctrl_c = monitor_term_sinal();
    let started = Instant::now();

    thread::scope(|pool| {
        let requests = params.get_steps(&core_info);
        let transfer = &params.transfer;

        let readers_channel = transfer.readers * 4;
        let writers_channel = transfer.writers * 3;

        let (generator, sequence) = bounded::<Step>(readers_channel);
        let (sender, receiver) = bounded::<Documents>(writers_channel);
        let (progress, reporter) = bounded::<u64>(transfer.writers);

        pool.spawn(|_| {
            start_querying_core(requests, generator, &ctrl_c);
            debug!("Finished generator thread");
        });

        for ir in 0..transfer.readers {
            let producer = sender.clone();
            let iterator = sequence.clone();
            let reader = ir;
            let max_errors = params.transfer.max_errors;

            let thread_name = format!("Reader_{}", reader);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_retrieving_docs(reader, iterator, producer, must_match, max_errors);
                    debug!("Finished reader #{}", reader);
                })
                .unwrap();
        }
        drop(sequence);
        drop(sender);

        let output_pat = params.get_archive_pattern(core_info.num_found);

        for iw in 0..transfer.writers {
            let consumer = receiver.clone();
            let updater = progress.clone();

            let dir = params.transfer.dir.clone();
            let name = output_pat.clone();
            let max = params.archive_files;

            let writer = iw;
            let thread_name = format!("Writer_{}", writer);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_storing_docs(writer, dir, name, max, consumer, updater);
                    debug!("Finished writer #{}", writer);
                })
                .unwrap();
        }
        drop(receiver);
        drop(progress);

        retrieved =
            foreach_progress(reporter, num_retrieving, params.num_docs, params.options.is_quiet());
    })
    .unwrap();

    if ctrl_c.aborted() {
        raise("# Execution aborted by user!")
    } else {
        info!(
            "Dowloaded {} of {} documents in {:?}.",
            retrieved,
            num_retrieving,
            started.elapsed()
        );
        Ok(())
    }
}

// region Channels

fn start_querying_core(requests: Requests, generator: Sender<Step>, ctrl_c: &Arc<AtomicBool>) {
    for step in requests {
        let status = generator.send(step);
        if status.is_err() || ctrl_c.aborted() {
            break;
        }
    }
    drop(generator);
}

fn start_retrieving_docs(
    reader: usize, iterator: Receiver<Step>, producer: Sender<Documents>, must_match: u64,
    max_errors: usize,
) {
    let ctrl_c = monitor_term_sinal();
    let mut error_count = 0;

    let mut client = SolrClient::new();
    loop {
        let received = iterator.recv();
        if ctrl_c.aborted() {
            break;
        }
        let failed = match received {
            Ok(step) => retrieve_docs_from_solr(reader, &producer, step, &mut client, must_match),
            Err(_) => true,
        };
        if failed {
            if error_count < max_errors {
                error_count += 1;
            } else {
                break;
            }
        }
        if ctrl_c.aborted() {
            break;
        }
    }
    drop(producer);
}

fn retrieve_docs_from_solr(
    reader: usize, producer: &Sender<Documents>, step: Step, client: &mut SolrClient,
    must_match: u64,
) -> bool {
    let query_url = step.url.as_str();
    let response = fetch_docs_from_solr(reader, client, query_url, must_match);
    match response {
        Err(_) => true,
        Ok(content) => {
            let parsed = SolrCore::parse_docs_from_query(&content);
            match parsed {
                None => {
                    error!("Error in thread #{} parsing from solr query: {}", reader, query_url);
                    true
                }
                Some(json) => {
                    let docs = Documents { step, docs: json.to_string() };
                    let status = producer.send(docs);
                    status.is_err()
                }
            }
        }
    }
}

fn fetch_docs_from_solr(
    reader: usize, client: &mut SolrClient, query_url: &str, must_match: u64,
) -> Result<String, ()> {
    let mut times = 0;
    loop {
        let response = client.get_as_text(query_url);
        match response {
            Err(cause) => {
                error!("Error in thread #{} retrieving docs from solr: {}", reader, cause);
                return Err(());
            }
            Ok(content) => {
                if must_match > 0 {
                    match SolrCore::parse_num_found(&content) {
                        Ok(num_found) => {
                            trace!("#{} got num_found {} not {}", times, num_found, must_match);
                            if must_match != num_found.to_u64() && times < 13 {
                                times += 1;
                                wait(1);
                                continue;
                            }
                        }
                        Err(cause) => {
                            error!("Error in Solr response: {}", cause);
                            return Err(());
                        }
                    }
                }
                break Ok(content);
            }
        }
    }
}

fn start_storing_docs(
    writer: usize, dir: PathBuf, name: String, max: usize, consumer: Receiver<Documents>,
    progress: Sender<u64>,
) {
    let mut archiver = Archiver::write_on(&dir, &name, max);
    loop {
        let received = consumer.recv();
        match received {
            Ok(docs) => {
                let failed = archiver.write_documents(&docs);
                if let Err(cause) = failed {
                    error!("Error in thread #{} writing file into archive: {}", writer, cause);
                    break;
                }
                let status = progress.send(0);
                if status.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    drop(consumer);
}

// endregion

// end of file \\
