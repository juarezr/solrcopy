use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use reqwest::blocking::Client;

use super::args::Restore;
use super::helpers::*;
use super::fails::*;

pub fn put_content(params: &Restore, content: String) -> Result<(), BoxedError> {

    let url = params.get_update_url();

    // TODO: handle network error, timeout on posting

    http_post_to(&url, content)?;
    Ok(())
}

// region Http Client 

fn http_post_to(url: &str, content: String) -> Result<String, reqwest::Error> {
    
    // TODO: authentication

    let client = Client::new();
    let response = client.post(url)
        .headers(construct_headers())
        .body(content)
        .send()?;

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

        let parts: Vec<String> = vec!(
            self.options.url.with_suffix("/"),
            self.into.clone(),
            "/update?commit=true".to_string(),
        );
        parts.concat()
    }
}
