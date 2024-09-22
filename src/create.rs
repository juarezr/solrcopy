use super::{args::Execute, connection::SolrClient};
use log::{debug, info};

pub(crate) fn create_main(params: &Execute) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# CREATE {:?}", params);

    let core_name = params.options.core.clone();

    let json = "{ \"create\": { \"name\": \"%s\", \"configSet\": \"%c\" } }";
    let config_set = "/opt/solr/server/solr/configsets/_default";
    let content = json.replace("%s", &core_name).replace("%c", config_set);

    let url = params.options.get_core_admin_v2_url();
    debug!("# POST {}:\n  {}", url, content);
    println!("# POST {}:\n  {}", url, content);

    let res = SolrClient::new().post_as_json(&url, &content)?;

    info!("Created the core {} in {}:\n  {}", core_name, url, res);

    Ok(())
}
