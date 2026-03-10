use super::helpers::{IntegerHelpers, StringHelpers, env_value};
use log::{debug, trace};
use std::time::Duration;
use std::{error::Error, fmt};
use ureq::Agent;

#[cfg(not(any(test, debug_assertions)))]
use super::helpers::wait;

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

    pub(crate) fn of(message: &str, error_code: u16) -> Self {
        SolrError { details: message.to_string(), code: Some(error_code) }
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

// region SolrInfo

#[derive(Debug)]
pub(crate) struct SolrInfo {
    pub version: i64,
    pub standalone: bool,
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

const CONTENT_TYPE: &str = "Content-Type";
const APPLICATION_JSON: &str = "application/json";
const APPLICATION_XML: &str = "application/xml";

#[cfg(debug_assertions)]
const SOLR_DEF_RETRIES: isize = 1;
#[cfg(not(debug_assertions))]
const SOLR_DEF_RETRIES: isize = 8;

#[cfg(not(any(test, debug_assertions)))]
const SOLR_WAIT_SECS: usize = 5;

impl SolrClient {
    pub(crate) fn new() -> Self {
        let retries = env_value(SOLR_COPY_RETRIES, SOLR_DEF_RETRIES);
        let timeout = Self::get_timeout();
        let duration = Option::from(Duration::from_secs(timeout));

        let builder = Agent::config_builder().timeout_global(duration).build();
        let client = builder.into();

        SolrClient { http: client, max_retries: retries.to_usize(), retry_count: 0 }
    }

    fn get_timeout() -> u64 {
        let def = if cfg!(debug_assertions) { 6 } else { SOLR_DEF_TIMEOUT };
        let timeout: isize = env_value(SOLR_COPY_TIMEOUT, def);
        timeout.to_u64()
    }

    pub(crate) fn get_as_json(&mut self, url: &str) -> Result<String, SolrError> {
        self.get_with_content_type(url, APPLICATION_JSON)
    }

    pub(crate) fn get_with_content_type(
        &mut self, url: &str, content_type: &str,
    ) -> Result<String, SolrError> {
        debug!("# curl --fail --location -X GET {}", url);
        loop {
            let req = self.http.get(url);
            let request = req.header(CONTENT_TYPE, content_type);
            let answer = request.call();
            let result = self.handle_response(answer);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    pub(crate) fn get_solr_info(&mut self, url: &str) -> Result<SolrInfo, SolrError> {
        let system_url = url.with_suffix("/").append("admin/info/system?wt=json");

        let system_json = self.get_as_json(&system_url)?;

        let parsed = serde_json::from_str::<serde_json::Value>(&system_json);
        let info = parsed.map_err(|e| SolrError::new(e.to_string(), 500))?;

        let version = info
            .get("lucene")
            .and_then(|lucene| lucene.get("solr-spec-version"))
            .and_then(|solr_impl_version| solr_impl_version.as_str())
            .ok_or_else(|| SolrError::of("Solr version not found in system info", 500))?;

        let parts: Vec<&str> = version.split('.').collect();
        let major = parts[0].parse::<i64>().map_err(|e| SolrError::new(e.to_string(), 500))?;

        let mode = info
            .get("mode")
            .and_then(|mode| mode.as_str())
            .ok_or_else(|| SolrError::of("Solr mode not found in system info", 500))?;

        Ok(SolrInfo { version: major, standalone: mode == "std" })
    }

    pub(crate) fn post_as_json(&mut self, url: &str, content: &str) -> Result<String, SolrError> {
        self.post_with_content_type(url, APPLICATION_JSON, content)
    }

    pub(crate) fn post_as_xml(&mut self, url: &str, content: &str) -> Result<String, SolrError> {
        self.post_with_content_type(url, APPLICATION_XML, content)
    }

    fn post_with_content_type(
        &mut self, url: &str, content_type: &str, content: &str,
    ) -> Result<String, SolrError> {
        trace!("POST as {} {}", content_type, url);
        loop {
            let req = self.http.post(url);
            let request = req.header("Content-Type", content_type);
            let answer = request.send(content);
            let result = self.handle_response(answer);
            match result {
                None => continue,
                Some(retrieved) => break retrieved,
            }
        }
    }

    fn handle_response(
        &mut self, answer: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
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
                    #[cfg(not(any(test, debug_assertions)))]
                    wait(SOLR_WAIT_SECS * self.retry_count);
                    None
                } else {
                    Some(Err(failure))
                }
            }
        }
    }

    fn decode_response(
        &mut self, answer: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    ) -> Result<String, SolrError> {
        match answer {
            Ok(mut response) => {
                let body = response.body_mut();
                let content = body.read_to_string();
                let status = response.status();
                match content {
                    Ok(text) => Ok(text),
                    Err(failed) => Err(SolrError::new(failed.to_string(), status.as_u16())),
                }
            }
            Err(failure) => match failure {
                ureq::Error::StatusCode(code) => Err(SolrError::new(failure.to_string(), code)),
                _ => Err(SolrError::from(failure.to_string())),
            },
        }
    }

    // region Helpers

    pub(crate) fn send_get_as_json(url: &str) -> Result<String, SolrError> {
        let mut con = SolrClient::new();
        con.get_as_json(url)
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
