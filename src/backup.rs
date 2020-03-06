use log::{debug, info};

use crate::args::Backup;

pub(crate) fn backup_main(parsed: Backup) -> Result<(), Box<dyn std::error::Error>> {
    debug!("  {:?}", parsed);

    let core_info = parsed.inspect_core()?;
    debug!("  {:?}", core_info);

    let mut archiver = parsed.get_writer()?;

    let steps = parsed.get_steps(&core_info);
    let range = steps.len();

    let docs = steps.retrieve();

    let working = docs.map(|response| {
        let filename = response.step.get_output_name();
        let docs = response.docs;
        archiver.write_file(filename, &docs).unwrap();
    });

    let report = crate::bars::get_wide_bar_for(working, range);

    let num = report.count();
    info!("Finished {} steps.", num);

    // TODO: split in multiple files of constant size
    archiver.close_archive()?;

    Ok(())
}
