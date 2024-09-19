use crossbeam_channel::{bounded, Receiver, Sender};
use crossbeam_utils::thread;
use log::{debug, error, info, trace};

use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::{path::Path, path::PathBuf, time::Instant};

use crate::{
    args::Restore, bars::*, connection::SolrClient, fails::*, helpers::*, ingest::*, state::*,
};

pub(crate) fn restore_main(params: &Restore) -> BoxedError {
    debug!("# RESTORE {:?}", params);

    let found = params.find_archives()?;

    if found.is_empty() {
        throw(format!(
            "Found no archives to restore from: {}\n note: try to specify the option --pattern \
             with the source core name",
            params.get_pattern()
        ))?;
    }

    let core = params.options.core.clone();
    info!(
        "Found {} zip archives in {:?} for updating into core {:?}",
        found.len(),
        params.transfer.dir,
        core
    );

    wait_with_progress(
        params.transfer.delay_before,
        &format!("Waiting before processing {}...", core),
    );

    pre_post_processing(params, false)?;

    let started = Instant::now();

    let updated = unzip_archives_and_send(params, &found)?;

    info!("Updated {} batches in solr core {} in {:?}.", updated, core, started.elapsed());

    pre_post_processing(params, true)?;

    if updated > 0 {
        wait_with_progress(params.transfer.delay_after, "Waiting after all processing...");
    }
    Ok(())
}

// region Processing

fn unzip_archives_and_send(params: &Restore, found: &[PathBuf]) -> BoxedResult<u64> {
    let doc_count = estimate_batch_count(found)?;
    let mut updated = 0;

    let core = params.options.core.clone();
    info!("Estimated {} batches for indexing in solr core {}", doc_count, core);

    thread::scope(|pool| {
        let transfer = &params.transfer;
        let readers_channel = transfer.readers * 2;
        let writers_channel = transfer.writers * 2;

        let (generator, sequence) = bounded::<&Path>(readers_channel.to_usize());
        let (sender, receiver) = bounded::<Docs>(writers_channel.to_usize());
        let (progress, reporter) = bounded::<u64>(transfer.writers.to_usize());

        pool.spawn(move |_| {
            start_listing_archives(found, generator);
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
                    start_reading_archive(reader, iterator, producer);
                    debug!("Finished reader #{}", reader);
                })
                .unwrap();
        }
        drop(sequence);
        drop(sender);

        let update_errors = Arc::new(AtomicU64::new(0));
        let update_hadler_url = params.get_update_url();
        debug!("Solr Update Handler: {}", update_hadler_url);

        for iw in 0..transfer.writers {
            let consumer = receiver.clone();
            let updater = progress.clone();

            let url = update_hadler_url.clone();
            let arcerr = Arc::clone(&update_errors);
            let merr = params.transfer.max_errors;
            let delay = params.transfer.delay_per_request;

            let writer = iw;
            let thread_name = format!("Writer_{}", writer);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_indexing_docs(writer, &url, consumer, updater, &arcerr, merr, delay);
                    debug!("Finished writer #{}", writer);
                })
                .unwrap();
        }
        drop(receiver);
        drop(progress);

        updated = foreach_progress(reporter, doc_count, 1, params.options.is_quiet());
    })
    .unwrap();

    finish_sending(params, updated)
}

fn finish_sending(params: &Restore, updated: u64) -> BoxedResult<u64> {
    let ctrl_c = monitor_term_sinal();

    if ctrl_c.aborted() {
        raise("# Execution aborted by user!")
    } else {
        if updated > 0 && !params.no_final_commit {
            // let params2 = Command { options: params.options };
            crate::commit::commit_main(&params.options.to_command())?;
        }
        Ok(updated)
    }
}

fn estimate_batch_count(found: &[PathBuf]) -> BoxedResult<u64> {
    // Estimate number of json files inside all zip files
    let zip_count = found.len();

    let first = found.first().unwrap();
    let file_count = ArchiveReader::get_archive_file_count(first);
    match file_count {
        None => throw(format!("Error opening archive: {:?}", first))?,
        Some(doc_count) => {
            let doc_total = doc_count * zip_count;
            Ok(doc_total.to_u64())
        }
    }
}

fn pre_post_processing(params: &Restore, enable: bool) -> BoxedResult<()> {
    let core = params.options.core.as_str();

    if params.disable_replication {
        let (verb, handler_path) = if enable {
            ("enabling", "replication?command=enablereplication")
        } else {
            ("disabling", "replication?command=disablereplication")
        };
        info!("Now {} replication in {}.", verb, core);

        let url = params.options.get_core_handler_url(handler_path);
        SolrClient::query_get_as_text(&url)?;
    }
    Ok(())
}

// endregion

// region Channels

fn start_listing_archives<'a>(found: &'a [PathBuf], generator: Sender<&'a Path>) {
    let archives = found.iter();
    for archive in archives {
        let status = generator.send(archive);
        if status.is_err() {
            break;
        }
    }
    drop(generator);
}

fn start_reading_archive(reader: u64, iterator: Receiver<&Path>, producer: Sender<Docs>) {
    let ctrl_c = monitor_term_sinal();

    loop {
        let received = iterator.recv();
        if received.is_err() || ctrl_c.aborted() {
            break;
        }
        let archive_path = received.unwrap();
        let failed = handle_reading_archive(reader, &producer, archive_path, &ctrl_c);
        if failed || ctrl_c.aborted() {
            break;
        }
    }
    drop(producer);
}

fn handle_reading_archive(
    reader: u64, producer: &Sender<Docs>, archive_path: &Path, ctrl_c: &Arc<AtomicBool>,
) -> bool {
    let zip_name: String = get_filename(archive_path).unwrap();
    trace!("Reading zip archive: {}", zip_name);
    let can_open = ArchiveReader::create_reader(archive_path);
    match can_open {
        Ok(archive_reader) => {
            for (entry_name, entry_contents) in archive_reader {
                trace!("  Uncompressing json: '{}' from '{}'", entry_name, zip_name);

                let docs = Docs::new(zip_name.clone(), entry_name, entry_contents);
                let status = producer.send(docs);
                if status.is_err() || ctrl_c.aborted() {
                    return true;
                }
            }
            false
        }
        Err(cause) => {
            error!("Error in thread #{} while reading docs in zip: {}", reader, cause);
            true
        }
    }
}

fn start_indexing_docs(
    writer: u64, url: &str, consumer: Receiver<Docs>, progress: Sender<u64>,
    error_count: &Arc<AtomicU64>, max_errors: u64, delay: u64,
) {
    let ctrl_c = monitor_term_sinal();

    let mut client = SolrClient::new();
    loop {
        let received = consumer.recv();
        if received.is_err() || ctrl_c.aborted() {
            break;
        }
        let docs = received.unwrap();
        let failed =
            send_to_solr(docs, writer, url, &mut client, &progress, error_count, max_errors);
        if failed || ctrl_c.aborted() {
            break;
        } else if delay > 0 {
            wait_by(delay.to_usize());
        }
    }
    drop(consumer);
}

fn send_to_solr(
    docs: Docs, writer: u64, url: &str, client: &mut SolrClient, progress: &Sender<u64>,
    error_count: &Arc<AtomicU64>, max_errors: u64,
) -> bool {
    let failed = client.post_as_json(url, docs.json.as_str());
    if let Err(cause) = failed {
        let current = error_count.fetch_add(1, Ordering::SeqCst);
        error!(
            "Error #{}/{} in thread #{} when indexing solr core:\n{}{:?}",
            current, max_errors, writer, cause, docs
        );
        current > max_errors
    } else {
        let status = progress.send(0);
        status.is_err()
    }
}

// endregion

#[cfg(test)]
mod tests {
    use crate::{args::*, fails::*};
    use pretty_assertions::assert_eq;

    impl Commands {
        pub fn put(&self) -> BoxedResult<&Restore> {
            match &self {
                Self::Restore(puts) => Ok(&puts),
                _ => raise("command must be 'restore' !"),
            }
        }
    }

    #[test]
    fn check_restore_pattern() {
        let parsed = Cli::mockup_args_restore();
        let puts = parsed.put().unwrap();
        let wilcard = puts.get_pattern();
        assert_eq!(wilcard.ends_with(".zip"), true);
    }

    #[test]
    fn check_restore_iterator() {
        let parsed = Cli::mockup_args_restore();
        let puts = parsed.put().unwrap();

        for zip in puts.find_archives().unwrap() {
            println!("{:?}", zip);
            let path = zip.to_str().unwrap();
            assert_eq!(path.ends_with(".zip"), true);
        }
    }
}
