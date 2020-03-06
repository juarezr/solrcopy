use zip::ZipArchive;

use log::debug;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::args::Restore;
use crate::connection::http_post_as_json;
use crate::fails::*;
use crate::helpers::*;

pub(crate) fn restore_main(params: Restore) -> Result<(), Box<dyn std::error::Error>> {
    debug!("  {:?}", params);

    let found = params.find_archives()?;

    if found.is_empty() {
        throw(format!("Found no archives to restore from: {}\n note: try to specify the option --pattern with the source core name", params.get_pattern()))?;
    }
    unzip_archives(params, found)
}

use crate::bars::*;

fn unzip_archives(params: Restore, found: Vec<PathBuf>) -> Result<(), BoxedError> {
    let zip_count = found.len().to_u64();
    let update_hadler_url = params.get_update_url();
    let barp = new_wide_bar(zip_count);

    for path in found {
        let zipfile = File::open(&path)?;
        let mut archive = ZipArchive::new(zipfile)?;
        let file_count = archive.len();

        let step_count = file_count.to_u64() * zip_count;
        barp.set_length(step_count);

        for i in 0..file_count {
            let mut compressed = archive.by_index(i).unwrap();
            let mut buffer = String::new();
            compressed.read_to_string(&mut buffer)?;

            put_content(&update_hadler_url, buffer)?;

            barp.inc(1);
        }
    }
    barp.finish();
    Ok(())
}

fn put_content(update_hadler_url: &str, content: String) -> Result<(), BoxedError> {
    // TODO: handle network error, timeout on posting
    http_post_as_json(&update_hadler_url, content)?;
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
