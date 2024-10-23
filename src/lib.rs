mod error;
mod find_similar;
mod get_contents;
mod search;

use std::{fmt::Debug, sync::Arc};

use anyhow::{anyhow, Result};
use futures::TryStreamExt;
use http_client::{
    http::{header::CONTENT_TYPE, Method, Request, Response},
    AsyncBody, HttpClient, RequestBuilderExt,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{de::DeserializeOwned, Serialize};

pub use crate::{error::*, find_similar::*, get_contents::*, search::*};

pub const BASE_URL: &str = "https://api.exa.ai";
pub const API_KEY_HEADER: &str = "x-api-key";

pub struct Exa {
    http_client: Arc<dyn HttpClient>,
    api_key: SecretString,
    base_url: String,
}

pub struct ExaBuilder {
    http_client: Option<Arc<dyn HttpClient>>,
    api_key: Option<SecretString>,
    base_url: Option<String>,
}

impl Exa {
    pub fn builder() -> ExaBuilder {
        ExaBuilder {
            http_client: None,
            api_key: None,
            base_url: None,
        }
    }

    pub(crate) async fn post<P, S, D>(&self, path: P, request: S) -> Result<D, ExaError>
    where
        P: Into<String>,
        S: Serialize + Debug,
        D: DeserializeOwned,
    {
        let response = self
            .http_client
            .send(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("{}{}", self.base_url, path.into()))
                    .header(CONTENT_TYPE, "application/json")
                    .header(API_KEY_HEADER, self.api_key.expose_secret())
                    .json(request)?,
            )
            .await?;

        handle_response(response).await
    }
}

async fn handle_response<D>(response: Response<AsyncBody>) -> Result<D, ExaError>
where
    D: DeserializeOwned,
{
    let status = response.status();

    if !status.is_success() {
        let body: Vec<Vec<u8>> = response.into_body().try_collect().await?;
        let body = body.into_iter().flatten().collect::<Vec<u8>>();
        let payload = serde_json::from_slice::<HttpErrorPayload>(&body).unwrap();
        return Err(ExaError::HttpError(HttpError {
            status: status.into(),
            payload,
        }));
    }

    let body: Vec<Vec<u8>> = response.into_body().try_collect().await?;
    let body = body.into_iter().flatten().collect::<Vec<u8>>();
    let response = serde_json::from_slice::<D>(&body).unwrap();
    Ok(response)
}

impl ExaBuilder {
    pub fn with_http_client(mut self, http_client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn with_api_key<S>(mut self, api_key: S) -> Self
    where
        S: AsRef<str>,
    {
        self.api_key = Some(api_key.as_ref().to_string().into());
        self
    }

    pub fn with_base_url<S>(mut self, base_url: S) -> Self
    where
        S: AsRef<str>,
    {
        self.base_url = Some(base_url.as_ref().into());
        self
    }

    pub fn build(self) -> Result<Exa> {
        Ok(Exa {
            http_client: self.http_client.ok_or_else(|| anyhow!("http client is required"))?,
            api_key: self.api_key.or_else(|| std::env::var("EXA_API_KEY").ok().map(SecretString::new))
                .ok_or_else(|| anyhow!("API key is required. Set it explicitly or use the EXA_API_KEY environment variable"))?,
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
        })
    }
}
