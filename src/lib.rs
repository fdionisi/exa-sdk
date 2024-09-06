mod error;
mod find_similar;
mod get_contents;
mod search;

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{de::DeserializeOwned, Serialize};

pub use crate::{error::*, find_similar::*, get_contents::*, search::*};

pub const BASE_URL: &str = "https://api.exa.ai";
pub const API_KEY_HEADER: &str = "x-api-key";

pub struct Exa {
    client: reqwest::Client,
    api_key: SecretString,
    base_url: String,
}

pub struct ExaBuilder {
    api_key: Option<SecretString>,
    base_url: Option<String>,
}

impl Exa {
    pub fn builder() -> ExaBuilder {
        ExaBuilder {
            api_key: None,
            base_url: None,
        }
    }

    pub(crate) async fn post<P, S, D>(&self, path: P, request: S) -> Result<D, ExaError>
    where
        P: Into<String>,
        S: Serialize,
        D: DeserializeOwned,
    {
        let headers = self.build_headers();

        let response = self
            .client
            .post(format!("{}{}", self.base_url, path.into()))
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        handle_response(response).await
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            API_KEY_HEADER,
            HeaderValue::from_str(&self.api_key.expose_secret())
                .expect("couldn't create header value"),
        );
        headers
    }
}

async fn handle_response<D>(response: Response) -> Result<D, ExaError>
where
    D: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await?;
        dbg!(&text);
        let payload = serde_json::from_str::<HttpErrorPayload>(&text).unwrap();
        return Err(ExaError::HttpError(HttpError {
            status: status.as_u16(),
            payload,
        }));
    }

    let response = response.json::<D>().await?;
    Ok(response)
}

impl ExaBuilder {
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn build(self) -> Result<Exa> {
        Ok(Exa {
            client: reqwest::Client::new(),
            api_key: self.api_key.or_else(|| std::env::var("EXA_API_KEY").ok().map(SecretString::new))
                .ok_or_else(|| anyhow!("API key is required. Set it explicitly or use the EXA_API_KEY environment variable"))?,
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
        })
    }
}
