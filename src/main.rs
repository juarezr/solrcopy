#![deny(warnings)] 

#[macro_use] 
extern crate lazy_static;

extern crate regex;
extern crate url;
extern crate reqwest;
extern crate zip;

mod fails;
mod helpers;
mod args;
mod steps;
mod fetch;
mod save;

use args::{Arguments, SolrCore};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let parsed = Arguments::parse_from_args()?;
    if parsed.verbose {
        // TODO: use a logger and combine with --verbose
        println!("  {:?}", parsed);
    }

    let core_info = SolrCore::inspect_core(&parsed)?;
    if parsed.verbose {
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

            if results.is_ok() {
                let rows = results.unwrap();
                archiver.write_file(&step, &rows).unwrap();
            }
            // TODO: print a warning about unbalanced shard in solr could configurations
        });

    // TODO: split in multiple files of constant size
    archiver.close_archive()?;

    Ok(())
}

// end of file
