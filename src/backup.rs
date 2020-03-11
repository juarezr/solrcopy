use log::{debug, error, info};
use std::path::PathBuf;
use std::time::Instant;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::crossbeam_channel::bounded;
use crossbeam::thread;

use crate::args::Backup;
use crate::bars::new_wide_bar;
use crate::fails::BoxedFailure;
use crate::helpers::*;
use crate::save::Archiver;
use crate::steps::{Documents, Requests, Step};

pub(crate) fn backup_main(parsed: Backup) -> BoxedFailure {
    debug!("  {:?}", parsed);

    let core_info = parsed.inspect_core()?;
    debug!("  {:?}", core_info);

    let num_found = core_info.num_found.to_u64();

    info!(
        "Starting retrieving {} documents from solr core {}.",
        num_found, parsed.from
    );

    let started = Instant::now();

    thread::scope(|pool| {
        let requests = parsed.get_steps(&core_info);

        let readers_channel = parsed.readers * 4;
        let writers_channel = parsed.writers * 3;

        let (generator, sequence) = bounded::<Step>(readers_channel);
        let (sender, receiver) = bounded::<Documents>(writers_channel);
        let (progress, reporter) = bounded::<u64>(parsed.writers);

        pool.spawn(|_| {
            start_querying_core(requests, generator);
        });

        for ir in 0..parsed.readers {
            let producer = sender.clone();
            let iterator = sequence.clone();
            let reader = ir;

            let thread_name = format!("Reader_{}", reader);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_retrieving_docs(reader, iterator, producer);
                })
                .unwrap();
        }
        drop(sequence);
        drop(sender);

        let output_pat = parsed.get_archive_pattern(core_info.num_found);

        for iw in 0..parsed.writers {
            let consumer = receiver.clone();
            let updater = progress.clone();

            let dir = parsed.into.clone();
            let name = output_pat.clone();

            let writer = iw;
            let thread_name = format!("Writer_{}", writer);
            pool.builder()
                .name(thread_name)
                .spawn(move |_| {
                    start_storing_docs(writer, dir, name, consumer, updater);
                })
                .unwrap();
        }
        drop(receiver);
        drop(progress);

        let perc_bar = new_wide_bar(num_found);
        for _ in reporter.iter() {
            perc_bar.inc(1);
        }
        drop(reporter);
    })
    .unwrap();

    info!(
        "Dowloaded {} documents in {:?}.",
        num_found,
        started.elapsed()
    );
    Ok(())
}

fn start_querying_core(requests: Requests, generator: Sender<Step>) {
    debug!("  Generating ");
    for step in requests {
        generator.send(step).unwrap();
    }
    drop(generator);
    debug!("  Finished Generating ");
}

fn start_retrieving_docs(reader: usize, iterator: Receiver<Step>, producer: Sender<Documents>) {
    debug!("  Producing #{}", reader);

    loop {
        let received = iterator.recv();
        if let Ok(step) = received {
            let retrieved = step.retrieve_docs();
            match retrieved {
                Ok(docs) => producer.send(docs).unwrap(),
                Err(cause) => {
                    error!("Error retrieving documents from solr: {}", cause);
                    break;
                }
            }
        } else {
            break;
        }
    }
    drop(producer);

    debug!("  Finished Producing #{}", reader);
}

fn start_storing_docs(
    writer: usize,
    dir: PathBuf,
    name: String,
    consumer: Receiver<Documents>,
    progress: Sender<u64>,
) {
    debug!("  Consuming #{}", writer);

    let mut archiver = Archiver::write_on(&dir, &name);
    loop {
        let received = consumer.recv();
        if let Ok(docs) = received {
            let failed = archiver.write_documents(&docs);
            if let Err(cause) = failed {
                error!("Error writing file into archive: {}", cause);
                break;
            }
            progress.send(0).unwrap();
        } else {
            break;
        }
    }
    drop(consumer);

    debug!("  Finished Consuming #{}", writer);
}
