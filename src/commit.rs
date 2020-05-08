use log::{debug, info};

use super::{args::Commit, connection::SolrClient, helpers::*};

pub(crate) fn commit_main(params: Commit) -> Result<(), Box<dyn std::error::Error>> {
    debug!("  {:?}", params);

    let url = params.get_update_url();

    let content = "{ \"commit\": {} } ";

    SolrClient::send_post_as_json(&url, content)?;

    info!("Commited ocuments in {}.", url);

    Ok(())
}

impl Commit {
    fn get_update_url(&self) -> String {
        #[rustfmt::skip]
        let parts: Vec<String> = vec![
            self.options.url.with_suffix("/"),
            self.options.core.clone(),
            "/update".to_string(),
        ];
        parts.concat()
    }
}
