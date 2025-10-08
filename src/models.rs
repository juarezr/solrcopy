// region Data Structures

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

// endregion
