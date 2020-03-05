use log::{debug, info};

use crate::args::Backup;
use crate::steps::SolrCore;

pub(crate) fn backup_main(parsed: Backup) -> Result<(), Box<dyn std::error::Error>> {
    debug!("  {:?}", parsed);

    let core_info = SolrCore::inspect_core(&parsed)?;
    debug!("  {:?}", core_info);

    let mut archiver = parsed.get_writer()?;

    let steps = parsed.get_steps(&core_info);
    let range = steps.len();

    let working = steps.map(|step| {
        let query_url = &step.url;
        let response = SolrCore::get_docs_from(&query_url);
        // TODO: retry on network errors and timeouts
        // TODO: print a warning about unbalanced shard in solr could configurations

        if let Ok(docs) = response {
            archiver.write_file(&step, &docs).unwrap();
        }
    });

    let report = crate::bars::get_wide_bar_for(working, range);

    let num = report.count();
    info!("Finished {} steps.", num);

    // TODO: split in multiple files of constant size
    archiver.close_archive()?;

    Ok(())
}
