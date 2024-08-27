mod error;
mod find_similar;
mod get_contents;
mod search;

use anyhow::{anyhow, Result};
use secrecy::SecretString;

pub use crate::{error::*, find_similar::*, get_contents::*, search::*};

pub const BASE_URL: &str = "https://api.exa.ai";

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
            api_key: self.api_key.ok_or_else(|| anyhow!("api_jey is required"))?,
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
        })
    }
}
