use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
};

use std::time::Duration;

use crate::helpers::wait;

// region Http Client

#[derive(Debug)]
pub struct SolrClient {
    http: Client,
    max_retries: usize,
}
    
// TODO: authentication, proxy, tls, etc...

impl SolrClient {
    pub fn new(max_retries: usize) -> Result<Self, reqwest::Error> {

        let timeout_var = option_env!("SOLR_COPY_TIMEOUT").unwrap_or("60");
        let timeout = timeout_var.parse::<u64>().unwrap();

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            // .basic_auth("admin", Some("good password"))
            .build()?;

        Ok(SolrClient {
            http: client,
            max_retries
        })
    }

    pub fn get_as_text(&mut self, url: &str) -> Result<String, reqwest::Error> {
        let retries = 0;

        loop {
            let response = self.http.get(url).send();
            match response {
                Ok(success) => {
                    let text = success.text()?;
                    break Ok(text)
                },
                Err(fail) => {
                    if fail.is_timeout() && retries < self.max_retries {
                        self.max_retries += 1;
                        wait(20);
                        continue;
                    }
                    break Err(fail)
                }
            }
        }
    }

    pub(crate) fn post_as_json(&mut self, url: &str, content: String) -> Result<String, reqwest::Error> {
        let retries = 0;

        loop {
            let response = self.http
                .post(url)
                .headers(construct_headers())
                .body(content.clone())
                .send();

            match response {
                Ok(success) => {
                    let text = success.text()?;
                    break Ok(text)
                },
                Err(fail) => {
                    if fail.is_timeout() && retries < self.max_retries {
                        self.max_retries += 1;
                        wait(20);
                        continue;
                    }
                    break Err(fail)
                }
            }
        }
    }
}

pub(crate) fn http_get_as_text(url: &str) -> Result<String, reqwest::Error> {
    let mut con = SolrClient::new(0)?;
    con.get_as_text(url)
}

pub(crate) fn http_post_as_json(url: &str, content: String) -> Result<String, reqwest::Error> {
    let mut con = SolrClient::new(0)?;
    con.post_as_json(url, content)
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

// endregion
