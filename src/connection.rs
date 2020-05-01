use ureq;

use std::{error::Error, fmt};

use crate::helpers::*;

// region Http Client

#[derive(Debug)]
pub struct SolrClient {
    http: ureq::Agent,
    max_retries: usize,
    retry_count: usize,
}

#[derive(Debug)]
pub struct SolrError {
    status: String,
    response: String,
}

impl SolrError {
    pub fn new(message: String, body: String) -> Self {
        SolrError { status: message, response: body }
    }

    fn say(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)
    }
}

impl fmt::Display for SolrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.say(f)
    }
}

// impl fmt::Debug for SolrError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.say(f)
//     }
// }

impl Error for SolrError {
    fn description(&self) -> &str {
        &self.status
    }
}

// TODO: authentication, proxy, etc...

const SOLR_COPY_TIMEOUT: &str = "SOLR_COPY_TIMEOUT";
const SOLR_COPY_RETRIES: &str = "SOLR_COPY_RETRIES";

impl SolrClient {
    pub fn new() -> Self {
        let retries = env_value(SOLR_COPY_RETRIES, 4);
        let client = ureq::agent()
            // .basic_auth("admin", Some("good password"))
            .build();

        SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 }
    }

    fn get_timeout() -> u64 {
        let timeout: isize = env_value(SOLR_COPY_TIMEOUT, 60);
        timeout.to_u64() * 1000
    }

    fn set_timeout(builder: &mut ureq::Request) -> &mut ureq::Request {
        let timeout = Self::get_timeout();
        builder
            .timeout_connect(timeout)
            .timeout_read(Self::get_timeout())
            .timeout_write(Self::get_timeout())
    }

    pub fn get_as_text(&mut self, url: &str) -> Result<String, SolrError> {
        let mut builder = self.http.get(url);
        let request = Self::set_timeout(&mut builder);
        let response = request.call();
        Self::handle_response(response)
    }

    pub fn post_as_json(&mut self, url: &str, content: String) -> Result<String, SolrError> {
        let mut builder = self.http.post(url);
        let req = Self::set_timeout(&mut builder);
        let request = req.set("Content-Type", "application/json");
        loop {
            let response = request.send_string(&content);
            let result = Self::handle_response(response);
            match result {
                Ok(returned) => {
                    if self.retry_count > 0 {
                        self.retry_count -= 1;
                    }
                    break Ok(returned);
                }
                Err(failed) => {
                    // TODO:  let is_timeout = response.is_timeout();
                    let retry = self.retry_count < self.max_retries;
                    if !retry {
                        break Err(failed);
                    }
                    self.retry_count += 1;
                    wait(20);
                }
            }
        }
    }

    fn handle_response(response: ureq::Response) -> Result<String, SolrError> {
        if response.error() {
            if response.synthetic() {
                let cause = response.synthetic_error().as_ref().unwrap();
                let msg = cause.status_text().to_string();
                let text = cause.body_text();
                Err(SolrError::new(msg, text))
            } else {
                let message = response.status_line().to_string();
                let body = response.into_string().unwrap();
                Err(SolrError::new(message, body))
            }
        } else {
            let body = response.into_string().unwrap();
            Ok(body)
        }
    }
}

pub fn http_get_as_text(url: &str) -> Result<String, SolrError> {
    let mut con = SolrClient::new();
    con.get_as_text(url)
}

pub fn http_post_as_json(url: &str, content: String) -> Result<String, SolrError> {
    let mut con = SolrClient::new();
    con.post_as_json(url, content)
}

// endregion
