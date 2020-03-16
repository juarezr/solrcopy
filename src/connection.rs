use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
};

use std::time::Duration;

use crate::helpers::*;

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
    pub fn new() -> Result<Self, reqwest::Error> {
        let timeout = env_value(SOLR_COPY_TIMEOUT, 60);
        let retries = env_value(SOLR_COPY_RETRIES, 4);

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout.to_u64()))
            // .basic_auth("admin", Some("good password"))
            .build()?;

        Ok(SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 })
    }

    pub fn get_as_text(&mut self, url: &str) -> Result<String, reqwest::Error> {
        let request = self.http.get(url);
        self.handle_request(&request)
    }

    pub fn post_as_json(&mut self, url: &str, content: String) -> Result<String, reqwest::Error> {
        let request = self.http.post(url).headers(json_headers()).body(content);
        self.handle_request(&request)
    }

    fn handle_request(
        &mut self, builder: &reqwest::blocking::RequestBuilder,
    ) -> Result<String, reqwest::Error> {
        loop {
            let request = builder.try_clone().unwrap();
            let result = request.send();
            let (retry, response) = match result {
                Ok(content) => self.handle_response(content),
                Err(failure) => self.handle_timeout(failure),
            };
            if retry {
                continue;
            }
            break response;
        }
    }

    fn handle_response(
        &mut self, response: reqwest::blocking::Response,
    ) -> (bool, Result<String, reqwest::Error>) {
        if self.retry_count > 0 {
            self.retry_count -= 1;
        }
        let resp = match response.error_for_status() {
            Ok(success) => match success.text() {
                Ok(retrieved) => Ok(retrieved),
                Err(parsing) => Err(parsing),
            },
            Err(cause) => Err(cause),
        };
        (false, resp)
    }

    fn handle_timeout(
        &mut self, failure: reqwest::Error,
    ) -> (bool, Result<String, reqwest::Error>) {
        let retry = failure.is_timeout() && self.retry_count < self.max_retries;
        if retry {
            self.retry_count += 1;
            wait(20);
        }
        (retry, Err(failure))
    }
}

pub fn http_get_as_text(url: &str) -> Result<String, reqwest::Error> {
    let mut con = SolrClient::new()?;
    con.get_as_text(url)
}

pub fn http_post_as_json(url: &str, content: String) -> Result<String, reqwest::Error> {
    let mut con = SolrClient::new()?;
    con.post_as_json(url, content)
}

fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

// endregion
