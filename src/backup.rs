use super::args::Backup;
use super::steps::SolrCore;

pub(crate) fn backup_main(parsed: Backup) -> Result<(), Box<dyn std::error::Error>> {
    if parsed.options.verbose {
        // TODO: use a logger and combine with --verbose
        println!("  {:?}", parsed);
    }

    let core_info = SolrCore::inspect_core(&parsed)?;
    if parsed.options.verbose {
        println!("  {:?}", core_info);
    }

    let mut archiver = parsed.get_writer()?;

    let steps = parsed.get_steps(&core_info);

    // TODO: refactor progress and finish it
    let items = steps.with_progress();

    items.for_each(|step| {
        let query_url = &step.url;
        let response = SolrCore::get_docs_from(&query_url);
        // TODO: retry on network errors and timeouts
        // TODO: print a warning about unbalanced shard in solr could configurations

        if let Ok(docs) = response {
            archiver.write_file(&step, &docs).unwrap();
        }
    });

    // TODO: split in multiple files of constant size
    archiver.close_archive()?;

    Ok(())
}
