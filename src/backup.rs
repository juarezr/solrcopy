use crossbeam::{
    channel::{Receiver, Sender},
    crossbeam_channel::bounded,
    thread,
};
use log::{debug, error, info};

use std::{path::PathBuf, time::Instant};

use crate::{
    args::Backup,
    bars::new_wide_bar,
    fails::BoxedFailure,
    helpers::*,
    save::Archiver,
    steps::{Documents, Requests, Step},
};

pub(crate) fn backup_main(params: Backup) -> BoxedFailure {
    debug!("  {:?}", params);

    let core_info = params.inspect_core()?;
    debug!("  {:?}", core_info);

    let num_found = core_info.num_found.to_u64();

    info!("Starting retrieving {} documents from solr core {}.", num_found, params.from);

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
            start_querying_core(requests, generator);
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
        perc_bar.finish_and_clear();
        drop(reporter);
    })
    .unwrap();

    info!("Dowloaded {} documents in {:?}.", num_found, started.elapsed());
    Ok(())
}

// region Channels

fn start_querying_core(requests: Requests, generator: Sender<Step>) {
    for step in requests {
        generator.send(step).unwrap();
    }
    drop(generator);
}

fn start_retrieving_docs(reader: usize, iterator: Receiver<Step>, producer: Sender<Documents>) {
    loop {
        let received = iterator.recv();
        if let Ok(step) = received {
            let retrieved = step.retrieve_docs();
            match retrieved {
                Ok(docs) => producer.send(docs).unwrap(),
                Err(cause) => {
                    error!("Error in thread #{} retrieving documents from solr: {}", reader, cause);
                    break;
                }
            }
        } else {
            break;
        }
    }
    drop(producer);
}

fn start_storing_docs(
    writer: usize, dir: PathBuf, name: String, consumer: Receiver<Documents>, progress: Sender<u64>,
) {
    let mut archiver = Archiver::write_on(&dir, &name);
    loop {
        let received = consumer.recv();
        if let Ok(docs) = received {
            let failed = archiver.write_documents(&docs);
            if let Err(cause) = failed {
                error!("Error in thread #{} writing file into archive: {}", writer, cause);
                break;
            }
            progress.send(0).unwrap();
        } else {
            break;
        }
    }
    drop(consumer);
}

// endregion

// end of file \\
