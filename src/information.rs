use super::{args::Execute, connection::SolrClient};
use log::debug;

pub(crate) fn info_main(params: &Execute) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# INFORMATION {:?}", params);

    let mut client = SolrClient::new();
    let info = client.get_solr_info(&params.options.url)?;
    println!("# Solr {}:\n  {:?}", params.options.url, info);

    Ok(())
}
