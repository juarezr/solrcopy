use crossbeam_channel::{bounded, Receiver, Sender};
use crossbeam_utils::thread;
use log::{debug, error, info};

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
    debug!("  {:?}", core_info);

    let end_limit = params.get_docs_to_retrieve(&core_info);
    let num_retrieving = end_limit - params.skip;
    let num_found = core_info.num_found.to_u64();
    let mut retrieved = 0;

    info!(
        "Starting retrieving between {} and {} from {} documents of solr core {}.",
        params.skip + 1,
        end_limit,
        num_found,
        params.from
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

            let thread_name = format!("Reader_{}", reader);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_retrieving_docs(reader, iterator, producer);
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

            let dir = params.into.clone();
            let name = output_pat.clone();
            let max = params.max_files;

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

        retrieved = foreach_progress(
            reporter,
            num_retrieving,
            params.docs_per_step,
            params.options.is_quiet(),
        );
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

fn start_retrieving_docs(reader: usize, iterator: Receiver<Step>, producer: Sender<Documents>) {
    let ctrl_c = monitor_term_sinal();

    let mut client = SolrClient::new();
    loop {
        let received = iterator.recv();
        if ctrl_c.aborted() {
            break;
        }
        let failed = match received {
            Ok(step) => handle_received_batch(reader, &producer, step, &mut client),
            Err(_) => true,
        };
        if failed || ctrl_c.aborted() {
            break;
        }
    }
    drop(producer);
}

fn handle_received_batch(
    reader: usize, producer: &Sender<Documents>, step: Step, client: &mut SolrClient,
) -> bool {
    let src = &step.url;
    let response = client.get_as_text(src);
    match response {
        Err(cause) => {
            error!("Error in thread #{} retrieving docs from solr: {}", reader, cause);
            true
        }
        Ok(content) => {
            // TODO: print a warning about unbalanced shard in solr could configurations
            let parsed = SolrCore::parse_docs_from_query(&content);
            match parsed {
                None => {
                    error!("Error in thread #{} parsing from solr query: {}", reader, src);
                    true
                }
                Some(json) => {
                    // TODO: pass &str instead of String
                    let docs = Documents { step, docs: json.to_string() };
                    let status = producer.send(docs);
                    status.is_err()
                }
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
