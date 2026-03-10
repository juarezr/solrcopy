use super::{args::Execute, connection::SolrClient};
use log::{debug, info};

pub(crate) fn info_main(params: &Execute) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# INFORMATION {:?}", params);

    let mut client = SolrClient::new();
    let info = client.get_solr_info(&params.options.url)?;

    info!(
        "# {{ url: '{}', version: {}, standalone: {} }}",
        params.options.url, info.version, info.standalone
    );
    Ok(())
}
