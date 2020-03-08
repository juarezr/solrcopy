use log::{debug, info};

use std::path::PathBuf;

use crate::args::Restore;
use crate::bars::*;
use crate::fails::*;
use crate::helpers::*;
use crate::ingest::*;

pub(crate) fn restore_main(params: Restore) -> Result<(), BoxedError> {
    debug!("  {:?}", params);

    let found = params.find_archives()?;

    if found.is_empty() {
        throw(format!("Found no archives to restore from: {}\n note: try to specify the option --pattern with the source core name", 
            params.get_pattern()))?;
    }
    unzip_archives(params, found)
}

fn unzip_archives(params: Restore, found: Vec<PathBuf>) -> Result<(), BoxedError> {
    // reading from the zip archives and updating solr core

    let zip_count = found.len().to_u64();
    let barp = new_wide_bar(zip_count);

    let archives = load_all_archives_for(found);

    let estimated = archives.inspect(|reader| {
        let file_count = reader.archive.len();
        let step_count = file_count.to_u64() * zip_count;
        barp.set_length(step_count);
    });

    let documents = read_all_documents(estimated);

    let update_hadler_url = params.get_update_url();

    let responses = documents.map(|doc| post_content(&update_hadler_url, doc));

    let report = responses.inspect(|_| barp.inc(1));

    let num = report.count();
    info!("Finished updating documents in {} steps.", num);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::args::*;
    use crate::fails::*;

    impl Arguments {
        pub fn put(&self) -> Result<&Restore, BoxedError> {
            match &self {
                Self::Restore(puts) => Ok(&puts),
                _ => raise("command must be 'restore' !"),
            }
        }
    }

    #[test]
    fn check_restore_pattern() {
        let parsed = Arguments::mockup_args_restore();
        let puts = parsed.put().unwrap();
        let wilcard = puts.get_pattern();
        assert_eq!(wilcard.ends_with(".zip"), true);
    }

    #[test]
    fn check_restore_iterator() {
        let parsed = Arguments::mockup_args_restore();
        let puts = parsed.put().unwrap();

        for zip in puts.find_archives().unwrap() {
            println!("{:?}", zip);
            let path = zip.to_str().unwrap();
            assert_eq!(path.ends_with(".zip"), true);
        }
    }
}
