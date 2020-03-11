use log::{debug, error};
use std::path::PathBuf;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::crossbeam_channel::bounded;
use crossbeam::thread;

use crate::args::Backup;
use crate::fails::BoxedFailure;
use crate::helpers::*;
use crate::save::Archiver;
use crate::steps::{Documents, Requests, Step};

// use crate::bars::get_wide_bar_for;

pub(crate) fn backup_main(parsed: Backup) -> BoxedFailure {
    debug!("  {:?}", parsed);

    let core_info = parsed.inspect_core()?;
    debug!("  {:?}", core_info);

    let output_pat = parsed.get_archive_pattern(core_info.num_found);

    let channel_size = parsed.readers * 4;
    // debug!("  Channels: {} Output: {:?} Pattern: {}", channel_size, output_dir, output_pat);

    thread::scope(|pool| {
        let requests = parsed.get_steps(&core_info);

        let (generator, sequence) = bounded::<Step>(channel_size);
        let (sender, receiver) = bounded::<Documents>(channel_size);

        pool.spawn(|_| {
            start_querying_core(requests, generator);
        });

        for ir in 0..parsed.readers {
            let producer = sender.clone();
            let iterator = sequence.clone();
            let reader = ir.clone();

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

        for iw in 0..parsed.writers {
            let consumer = receiver.clone();
            let writer = iw.clone();
            let dir = parsed.into.clone();
            let name = output_pat.clone();
            let thread_name = format!("Writer_{}", writer);
            pool.builder()
                .name(thread_name)
                .spawn(move|_| {
                    start_storing_docs(writer, dir, name, consumer);
                })
                .unwrap();
        }
        drop(receiver);
    })
    .unwrap();

    // let archiver = parsed.get_writer(core_info.num_found)?;

    // let range = parsed.get_steps_count(core_info.num_found);

    // let docs = steps
    //     .flat_map(Step::retrieve_docs)
    //     .store_documents(archiver);

    // let report = get_wide_bar_for(docs, range);

    // let num = report.count();
    // info!(
    //     "Finished retrieving {} documents in {} steps.",
    //     core_info.num_found, num
    // );

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

fn start_storing_docs(writer: usize, dir: PathBuf, name: String, consumer: Receiver<Documents>) {
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
        } else {
            break;
        }
    }
    drop(consumer);

    debug!("  Finished Consuming #{}", writer);
}
