use super::{args::Execute, connection::SolrClient};
use log::{debug, info};

pub(crate) fn create_main(params: &Execute) -> Result<(), Box<dyn std::error::Error>> {
    debug!("# CREATE {:?}", params);

    let mut client = SolrClient::new();
    let information = client.get_solr_info(&params.options.url)?;
    info!("# Solr {}:  {:?}", params.options.url, information);

    let core_name = params.options.core.clone();

    let std8 =
        r#"{ "create": { "name": "%s", "configSet": "/var/solr/data/configsets/_default" } }"#;
    let std9 = r#"{ "create": { "name": "%s", "configSet": "_default" } }"#;
    let stdx = r#"{ "name": "%s", "configSet": "_default" }"#;
    let cld9 = r#"{ "name": "%s", "config": "_default", "numShards": 1 }"#;

    let std = match information.version {
        8 => std8,
        9 => std9,
        _ => stdx,
    };
    let json = if information.standalone { std } else { cld9 };
    let content = json.replace("%s", &core_name);

    let api_url = if information.standalone { "api/cores" } else { "api/collections" };
    let url = params.options.get_url_from(api_url);

    debug!("# POST {}:\n  {}", url, content);
    println!("# POST {}:\n  {}", url, content);

    let res = client.post_as_json(&url, &content)?;

    println!("Created the core {} in {}:\n  {}", core_name, url, res);

    Ok(())
}
