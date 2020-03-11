use log::{debug, error};
use std::path::PathBuf;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::crossbeam_channel::unbounded;
use crossbeam::thread::scope;

use crate::args::Backup;
use crate::fails::BoxedFailure;
use crate::save::Archiver;
use crate::steps::{Documents, Step, Steps};

// use crate::bars::get_wide_bar_for;
// use crate::save::DocumentIterator;

pub(crate) fn backup_main(parsed: Backup) -> BoxedFailure {
    debug!("  {:?}", parsed);

    let core_info = parsed.inspect_core()?;
    debug!("  {:?}", core_info);

    let output_dir = parsed.into.clone();
    let output_pat = parsed.get_archive_pattern(core_info.num_found);

    scope(|pool| {
        let steps = parsed.get_steps(&core_info);

        let (sender, receiver) = unbounded::<Documents>();

        let producer = sender.clone();
        pool.spawn(|_| {
            start_retrieving_docs(steps, producer);
        });

        let consumer = receiver.clone();
        pool.spawn(|_| {
            start_storing_docs(&output_dir, &output_pat, consumer);
        });

        drop(sender);
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

fn start_retrieving_docs(steps: Steps, producer: Sender<Documents>) {
    debug!("  Producing ");
    let responses = steps.flat_map(Step::retrieve_docs);
    for docs in responses {
        producer.send(docs).unwrap();
    }
    drop(producer);
    debug!("  Finished Producing ");
}

fn start_storing_docs(output_dir: &PathBuf, output_pattern: &str, consumer: Receiver<Documents>) {
    debug!("  Consuming ");
    let mut archiver = Archiver::write_on(output_dir, output_pattern);
    loop {
        let received = consumer.recv();
        if let Ok(docs) = received {
            let failed = archiver.write_documents(&docs);
            if let Err(cause) = failed {
                // error!("Error writing file {} into archive: {}", docs.step.get_docs_filename(), cause);
                error!("Error writing file into archive: {}", cause);
                break;
            }
        } else {
            break;
        }
    }
    drop(consumer);
    debug!("  Finished Consuming ");
}
