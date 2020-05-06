#![allow(warnings)]
use std::{error::Error, fmt};

use crate::helpers::*;

// region SolrError

#[derive(Debug)]
pub struct SolrError {
    pub status: String,
    pub response: String,
}

impl SolrError {
    pub fn new(message: String, body: String) -> Self {
        SolrError { status: message, response: body }
    }

    fn say(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.response.is_empty() {
            write!(f, "{}", self.status)
        } else {
            write!(f, "{}\n{}", self.status, self.response)
        }
    }
}

impl fmt::Display for SolrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.say(f)
    }
}

impl Error for SolrError {
    fn description(&self) -> &str {
        &self.status
    }
}

// endregion

// region SolrClient

#[derive(Debug)]
pub struct SolrClient {
    http: ureq::Agent,
    max_retries: usize,
    retry_count: usize,
}

// TODO: authentication, proxy, etc...

const SOLR_COPY_TIMEOUT: &str = "SOLR_COPY_TIMEOUT";
const SOLR_COPY_RETRIES: &str = "SOLR_COPY_RETRIES";

const SOLR_DEF_TIMEOUT: isize = 60;
const SOLR_DEF_RETRIES: isize = 4;

impl SolrClient {
    pub fn new() -> Self {
        let retries = env_value(SOLR_COPY_RETRIES, SOLR_DEF_RETRIES);
        let client = ureq::agent()
            // .basic_auth("admin", Some("good password"))
            .build();

        SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 }
    }

    fn get_timeout() -> u64 {
        let def = if cfg!(debug_assertions) { 6 } else { SOLR_DEF_TIMEOUT };
        let timeout: isize = env_value(SOLR_COPY_TIMEOUT, def);
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
        loop {
            let response = request.call();
            let result = self.handle_response(response);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    pub fn post_as_json(&mut self, url: &str, content: &str) -> Result<String, SolrError> {
        let mut builder = self.http.post(url);
        let req = Self::set_timeout(&mut builder);
        let request = req.set("Content-Type", "application/json");
        loop {
            let response = request.send_string(content);
            let result = self.handle_response(response);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    fn handle_response(&mut self, response: ureq::Response) -> Option<Result<String, SolrError>> {
        let result = self.get_result_from(response);
        match result {
            Ok(retrieved) => {
                if self.retry_count > 0 {
                    self.retry_count -= 1;
                }
                Some(Ok(retrieved))
            }
            Err(failure) => {
                match self.handle_failure(failure) {
                    None => {
                        self.retry_count += 1;
                        // wait a little for the server recovering before retrying
                        wait(5 * self.retry_count.to_u64());
                        None
                    }
                    Some(failed) => Some(Err(failed)),
                }
            }
        }
    }

    fn get_result_from(
        &mut self, response: ureq::Response,
    ) -> Result<String, Result<ureq::Response, std::io::Error>> {
        if response.error() {
            Err(Ok(response))
        } else {
            match response.into_string() {
                Ok(body) => Ok(body),
                Err(read_error) => Err(Err(read_error)),
            }
        }
    }

    fn handle_failure(
        &mut self, failure: Result<ureq::Response, std::io::Error>,
    ) -> Option<SolrError> {
        let can_retry = self.retry_count < self.max_retries;
        match failure {
            Ok(response) => {
                if response.synthetic() {
                    Self::handle_synthetic_error(can_retry, response)
                } else {
                    Self::handle_solr_error(can_retry, response)
                }
            }
            Err(read_error) => Self::handle_receive_error(can_retry, read_error),
        }
    }

    fn handle_synthetic_error(can_retry: bool, response: ureq::Response) -> Option<SolrError> {
        let cause = response.synthetic_error().as_ref().unwrap();
        match cause {
            ureq::Error::ConnectionFailed(_) => Self::convert_synthetic_error(can_retry, cause),
            ureq::Error::Io(failure) => {
                let error_kind = failure.kind();
                match error_kind {
                    std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::NotConnected
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::Interrupted => {
                        Self::convert_synthetic_error(can_retry, cause)
                    }
                    _ => Self::convert_synthetic_error(can_retry, cause),
                }
            }
            _ => Self::convert_synthetic_error(false, cause),
        }
    }

    fn convert_synthetic_error(can_retry: bool, cause: &ureq::Error) -> Option<SolrError> {
        if can_retry {
            return None;
        }
        let msg = cause.status_text().to_string();
        let text = cause.body_text();
        Some(SolrError::new(msg, text))
    }

    fn handle_solr_error(can_retry: bool, response: ureq::Response) -> Option<SolrError> {
        // Retry on status 503 Service Temporarily Unavailable
        if can_retry && response.status() == 503 {
            return None;
        }
        let message = response.status_line().to_string();
        let body = response.into_string().unwrap();
        Some(SolrError::new(message, body))
    }

    fn handle_receive_error(can_retry: bool, error: std::io::Error) -> Option<SolrError> {
        if can_retry {
            return None;
        }
        let msg = error.to_string();
        let text = format!("{}", error);
        Some(SolrError::new(msg, text))
    }

    // region Helpers

    pub fn query_get_as_text(url: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.get_as_text(url)
    }

    pub fn send_post_as_json(url: &str, content: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.post_as_json(url, content)
    }

    // endregion
}

pub fn http_get_as_text(url: &str) -> Result<String, SolrError> {
    let mut con = SolrClient::new();
    con.get_as_text(url)
}

pub fn http_post_as_json(url: &str, content: &str) -> Result<String, SolrError> {
    let mut con = SolrClient::new();
    con.post_as_json(url, content)
}

// endregion
