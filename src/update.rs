use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};

use super::args::Restore;
use super::fails::*;
use super::helpers::*;

pub fn put_content(params: &Restore, content: String) -> Result<(), BoxedError> {
    let url = params.get_update_url();

    // TODO: handle network error, timeout on posting

    http_post_to(&url, content)?;
    Ok(())
}

// region Http Client

fn http_post_to(url: &str, content: String) -> Result<String, reqwest::Error> {
    let response = Client::new()
        .post(url)
        .headers(construct_headers())
        .body(content)
        .send()?;

    // TODO: authentication
    response.text()
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

// endregion

impl Restore {
    pub fn get_update_url(&self) -> String {
        let parts: Vec<String> = vec![
            self.options.url.with_suffix("/"),
            self.into.clone(),
            "/update".to_string(),
            self.commit.as_param("?"),
        ];
        parts.concat()
    }
}
