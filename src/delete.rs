use super::{args::Delete, connection::SolrClient, fails::BoxedResult};
use log::{debug, info};

pub(crate) fn delete_main(params: &Delete) -> BoxedResult<()> {
    debug!("# DELETE  {:?}", params);

    let url = params.options.get_update_url_with(params.flush.as_param("?").as_str());

    let content = format!("<delete><query>{}</query></delete>", params.query);

    SolrClient::send_post_as_xml(&url, &content)?;

    info!("Deleted documents in {}.", url);

    Ok(())
}
