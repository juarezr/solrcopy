use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
};

use std::time::Duration;

use crate::{fails::*, helpers::*};

// region Http Client

#[derive(Debug)]
pub struct SolrClient {
    http: Client,
    max_retries: usize,
    retry_count: usize,
}

// TODO: authentication, proxy, tls, etc...

const SOLR_COPY_TIMEOUT: &str = "SOLR_COPY_TIMEOUT";
const SOLR_COPY_RETRIES: &str = "SOLR_COPY_RETRIES";

impl SolrClient {
    pub fn new() -> BoxedResult<Self> {
        let timeout = env_value(SOLR_COPY_TIMEOUT, 60)?;
        let retries = env_value(SOLR_COPY_RETRIES, 4)?;

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout.to_u64()))
            // .basic_auth("admin", Some("good password"))
            .build()?;

        Ok(SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 })
    }

    pub fn get_as_text(&mut self, url: &str) -> BoxedResult<String> {
        loop {
            let response = self.http.get(url).send();
            match response {
                Ok(success) => {
                    if self.retry_count > 0 {
                        self.retry_count -= 1;
                    }
                    let text = success.text()?;
                    break Ok(text);
                }
                Err(fail) => {
                    if fail.is_timeout() && self.retry_count < self.max_retries {
                        self.retry_count += 1;
                        wait(20);
                        continue;
                    }
                    break rethrow(fail);
                }
            }
        }
    }

    pub(crate) fn post_as_json(&mut self, url: &str, content: String) -> BoxedResult<String> {
        let retries = 0;

        loop {
            let response =
                self.http.post(url).headers(construct_headers()).body(content.clone()).send();

            match response {
                Ok(success) => {
                    let text = success.text()?;
                    break Ok(text);
                }
                Err(fail) => {
                    if fail.is_timeout() && retries < self.max_retries {
                        self.max_retries += 1;
                        wait(20);
                        continue;
                    }
                    break rethrow(fail);
                }
            }
        }
    }
}

pub(crate) fn http_get_as_text(url: &str) -> BoxedResult<String> {
    let mut con = SolrClient::new()?;
    con.get_as_text(url)
}

pub(crate) fn http_post_as_json(url: &str, content: String) -> BoxedResult<String> {
    let mut con = SolrClient::new()?;
    con.post_as_json(url, content)
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

// endregion
