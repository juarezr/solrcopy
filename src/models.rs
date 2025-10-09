// region Data Structures

use clap::ValueEnum;

#[derive(Debug)]
pub(crate) struct Documents {
    pub step: Step,
    pub docs: String,
}

#[derive(Debug)]
pub(crate) struct Step {
    pub curr: u64,
    pub url: String,
}

#[derive(Debug)]
pub(crate) struct SolrCore {
    pub num_found: u64,
    pub fields: Vec<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum Compression {
    Stored,
    Zip,
    Zstd,
}

impl Compression {
    pub(crate) fn get_ext(&self) -> &str {
        if *self == Compression::Zstd { "zstd" } else { "zip" }
    }
}

// endregion
