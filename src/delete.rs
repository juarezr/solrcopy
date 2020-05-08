use log::{debug, info};

use super::{args::Delete, connection::SolrClient, fails::BoxedResult, helpers::*};

pub(crate) fn delete_main(params: Delete) -> BoxedResult<()> {
    debug!("  {:?}", params);

    let url = params.get_update_url();

    let content = format!("<delete><query>{}</query></delete>", params.query);

    SolrClient::send_post_as_xml(&url, &content)?;

    info!("Deleted documents in {}.", url);

    Ok(())
}

impl Delete {
    fn get_update_url(&self) -> String {
        #[rustfmt::skip]
        let parts: Vec<String> = vec![
            self.options.url.with_suffix("/"),
            self.options.core.clone(),
            "/update".to_string(),
            self.flush.as_param("?"),
        ];
        parts.concat()
    }
}
