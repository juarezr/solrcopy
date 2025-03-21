use super::helpers::{IntegerHelpers, env_value, wait};
use log::{debug, trace};
use std::time::Duration;
use std::{error::Error, fmt};

// region SolrError

#[derive(Debug)]
pub(crate) struct SolrError {
    pub details: String,
    pub code: Option<u16>,
}

impl SolrError {
    pub(crate) fn from(message: String) -> Self {
        SolrError { details: message, code: None }
    }

    pub(crate) fn new(message: String, error_code: u16) -> Self {
        SolrError { details: message, code: Some(error_code) }
    }

    fn say(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }

    pub(crate) fn is_fatal(&self) -> bool {
        if let Some(status) = self.code {
            return status >= 500;
        }
        false
    }
}

impl fmt::Display for SolrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.say(f)
    }
}

impl Error for SolrError {
    fn description(&self) -> &str {
        &self.details
    }
}

// endregion

// region SolrClient

#[derive(Debug)]
pub(crate) struct SolrClient {
    http: ureq::Agent,
    max_retries: usize,
    retry_count: usize,
}

// TODO: authentication, proxy, etc...

const SOLR_COPY_TIMEOUT: &str = "SOLR_COPY_TIMEOUT";
const SOLR_COPY_RETRIES: &str = "SOLR_COPY_RETRIES";

const SOLR_DEF_TIMEOUT: isize = 60;

#[cfg(debug_assertions)]
const SOLR_DEF_RETRIES: isize = 1;
#[cfg(not(debug_assertions))]
const SOLR_DEF_RETRIES: isize = 8;

#[cfg(debug_assertions)]
const SOLR_WAIT_SECS: usize = 1;
#[cfg(not(debug_assertions))]
const SOLR_WAIT_SECS: usize = 5;

impl SolrClient {
    pub(crate) fn new() -> Self {
        let retries = env_value(SOLR_COPY_RETRIES, SOLR_DEF_RETRIES);
        let timeout = Self::get_timeout();
        let duration = Duration::from_secs(timeout);

        let builder = ureq::AgentBuilder::new();
        let client = builder
            .timeout(duration)
            // .basic_auth("admin", Some("good password"))
            .build();

        SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 }
    }

    fn get_timeout() -> u64 {
        let def = if cfg!(debug_assertions) { 6 } else { SOLR_DEF_TIMEOUT };
        let timeout: isize = env_value(SOLR_COPY_TIMEOUT, def);
        timeout.to_u64()
    }

    pub(crate) fn get_as_text(&mut self, url: &str) -> Result<String, SolrError> {
        trace!("GET {}", url);
        loop {
            let request = self.http.get(url);
            let answer = request.call();
            let result = self.handle_response(answer);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    pub(crate) fn post_as_json(&mut self, url: &str, content: &str) -> Result<String, SolrError> {
        self.post_with_content_type(url, "application/json", content)
    }

    pub(crate) fn post_as_xml(&mut self, url: &str, content: &str) -> Result<String, SolrError> {
        self.post_with_content_type(url, "application/xml", content)
    }

    fn post_with_content_type(
        &mut self, url: &str, content_type: &str, content: &str,
    ) -> Result<String, SolrError> {
        trace!("POST as {} {}", content_type, url);
        loop {
            let req = self.http.post(url);
            let request = req.set("Content-Type", content_type);
            let answer = request.send_string(content);
            let result = self.handle_response(answer);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    fn handle_response(
        &mut self, answer: Result<ureq::Response, ureq::Error>,
    ) -> Option<Result<String, SolrError>> {
        let result = self.decode_response(answer);
        match result {
            Ok(content) => {
                if self.retry_count > 0 {
                    self.retry_count -= 1;
                }
                Some(Ok(content))
            }
            Err(failure) => {
                if !failure.is_fatal() && self.retry_count < self.max_retries {
                    self.retry_count += 1;
                    debug!(
                        "Retry {}/{}: Response Error: {}",
                        self.retry_count, self.max_retries, failure
                    );
                    // wait a little for the server recovering before retrying
                    wait(SOLR_WAIT_SECS * self.retry_count);
                    None
                } else {
                    Some(Err(failure))
                }
            }
        }
    }

    fn decode_response(
        &mut self, answer: Result<ureq::Response, ureq::Error>,
    ) -> Result<String, SolrError> {
        match answer {
            Ok(content) => {
                let status = content.status();
                let body = content.into_string();
                match body {
                    Ok(content) => Ok(content),
                    Err(failed) => Err(SolrError::new(failed.to_string(), status)),
                }
            }
            Err(failure) => match failure {
                ureq::Error::Status(code, response) => {
                    let msg = response.status_text();
                    Err(SolrError::new(msg.to_string(), code))
                }
                ureq::Error::Transport(transport) => {
                    let cause = transport.message().unwrap_or("Network failure.");
                    Err(SolrError::from(cause.to_string()))
                }
            },
        }
    }

    // region Helpers

    pub(crate) fn query_get_as_text(url: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.get_as_text(url)
    }

    pub(crate) fn send_post_as_json(url: &str, content: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.post_as_json(url, content)
    }

    pub(crate) fn send_post_as_xml(url: &str, content: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.post_as_xml(url, content)
    }

    // endregion
}

// endregion
