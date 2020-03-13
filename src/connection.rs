use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
};

// region Http Client

pub(crate) fn http_get_as_text(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(url)?;
    let content = response.text()?;
    Ok(content)
}

pub(crate) fn http_post_as_json(url: &str, content: String) -> Result<String, reqwest::Error> {
    #[rustfmt::skip]
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
