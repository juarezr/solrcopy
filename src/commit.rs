use log::{debug, info};

use super::{args::Command, connection::SolrClient};

pub(crate) fn commit_main(params: Command) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# COMMIT {:?}", params);

    let url = params.options.get_update_url();

    let content = "{ \"commit\": {} } ";

    SolrClient::send_post_as_json(&url, content)?;

    info!("Commited documents in {}.", url);

    Ok(())
}
