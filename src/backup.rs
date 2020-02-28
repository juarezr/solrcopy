
use super::args::Backup;
use super::steps::SolrCore;

pub (crate) fn backup_main(parsed: Backup) -> Result<(), Box<dyn std::error::Error>> {

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

            // TODO: retry on network errors and timeouts

            let query_url = &step.url;
            let results = SolrCore::get_rows_from(&query_url);

            if let Ok(rows) = results {
                archiver.write_file(&step, &rows).unwrap();
            }
            // TODO: print a warning about unbalanced shard in solr could configurations
        });

    // TODO: split in multiple files of constant size
    archiver.close_archive()?;

    Ok(())
}
