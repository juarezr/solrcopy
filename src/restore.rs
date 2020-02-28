use glob::{glob, Paths, PatternError};
use zip::ZipArchive;

use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use super::args::Restore;
use super::update::*;
use super::fails::*;

pub (crate) fn restore_main(params: Restore) -> Result<(), Box<dyn std::error::Error>> {

    if params.options.verbose {
        // TODO: use a logger and combine with --verbose
        println!("  {:?}", params);
    }
    let found = params.find_archives()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    unzip_archives(params, found)
}

fn unzip_archives(params: Restore, found: Vec<PathBuf>) -> Result<(), BoxedError> {
    
    // TODO: print progress
    
    for path in found {
        let zipfile = File::open(&path)?;
        let mut archive = ZipArchive::new(zipfile)?;
        let file_count = archive.len();
        
        for i in 0..file_count {
            let mut compressed = archive.by_index(i).unwrap();
            let mut buffer = String::new();
            compressed.read_to_string(&mut buffer)?;
            put_content(&params, buffer)?;
        }
    }
    Ok(())
}

impl Restore {

    fn find_archives(&self) -> Result<Paths, PatternError> {

        let wilcard = self.get_pattern();
        let found = glob(&wilcard)?;
        Ok(found)
    }
    
    fn get_pattern(&self) -> String {
        let wilcard = match &self.pattern  {
            Some(pat) => pat.to_string(),
            None => format!("{}*.zip", self.into),
        };
        let mut path = self.from.clone();
        path.push(wilcard);
        let res = path.to_str().unwrap();
        res.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::args::*;
    use crate::fails::*;

    impl Arguments {
        pub fn put(&self) ->  Result<&Restore, BoxedError> {
            match &self {
                Self::Restore(puts) => Ok(&puts),
                _ => raise("command must be 'restore' !"),
            }
        }
    }

    #[test]
    fn check_restore_pattern() {

        let parsed = Arguments::mockup_args_put();
        let puts = parsed.put().unwrap();
        let wilcard = puts.get_pattern();
        assert_eq!(wilcard.ends_with(".zip"), true);
    }

    #[test]
    fn check_restore_iterator() {

        let parsed = Arguments::mockup_args_put();
        let puts = parsed.put().unwrap();

        for zip in puts.find_archives().unwrap().filter_map(Result::ok) {
            println!("{:?}", zip);
            let path = zip.to_str().unwrap();
            assert_eq!(path.ends_with(".zip"), true);
        }   
    }

}    