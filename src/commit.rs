use super::{args::Execute, connection::SolrClient};
use log::{debug, info};

pub(crate) fn commit_main(params: &Execute) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# COMMIT {:?}", params);

    let url = params.options.get_update_url();

    let content = "{ \"commit\": {} } ";

    SolrClient::send_post_as_json(&url, content)?;

    info!("Commited documents in {}.", url);

    Ok(())
}
