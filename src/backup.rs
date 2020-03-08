use log::{debug, info};

use crate::args::Backup;
use crate::bars::get_wide_bar_for;
use crate::save::DocumentIterator;
use crate::steps::Step;

pub(crate) fn backup_main(parsed: Backup) -> Result<(), Box<dyn std::error::Error>> {
    debug!("  {:?}", parsed);

    let core_info = parsed.inspect_core()?;
    debug!("  {:?}", core_info);

    let archiver = parsed.get_writer(core_info.num_found)?;

    let steps = parsed.get_steps(&core_info);
    let range = steps.len();

    let docs = steps
        .flat_map(Step::retrieve_docs)
        .store_documents(archiver);

    let report = get_wide_bar_for(docs, range);

    let num = report.count();
    info!(
        "Finished retrieving {} documents in {} steps.",
        core_info.num_found, num
    );

    Ok(())
}
