use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::ExposeSecret;
use url::Url;

use crate::{Exa, ExaError, HttpError, HttpErrorPayload};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FindSimilarRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_crawl_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_crawl_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_text: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_text: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<crate::search::SearchContent>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct FindSimilarResponse {
    pub results: Vec<crate::search::SearchResult>,
}

impl FindSimilarRequest {
    pub fn new(url: String) -> Result<Self> {
        Url::parse(&url)?;
        Ok(Self {
            url,
            ..Default::default()
        })
    }
}

impl Exa {
    pub async fn find_similar(
        &self,
        request: FindSimilarRequest,
    ) -> Result<FindSimilarResponse, ExaError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&format!("Bearer {}", self.api_key.expose_secret()))
                .expect("couldn't create header value"),
        );

        let response = self
            .client
            .post(format!("{}/findSimilar", self.base_url))
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let payload = response.json::<HttpErrorPayload>().await?;
            return Err(ExaError::HttpError(HttpError {
                status: status.as_u16(),
                payload,
            }));
        }

        let find_similar_response = response.json::<FindSimilarResponse>().await?;
        Ok(find_similar_response)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mockito::Server as MockServer;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_find_similar() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/findSimilar")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "results": [{
                        "id": "test_id",
                        "title": "Test Title",
                        "url": "https://example.com",
                        "score": 0.95,
                        "publishedDate": "2023-01-01",
                        "author": "Test Author"
                    }]
                })
                .to_string(),
            )
            .create();

        let exa = Exa::builder()
            .api_key("test_key".to_string())
            .base_url(mock_url)
            .build()?;

        let request = FindSimilarRequest {
            url: "https://example.com".to_string(),
            num_results: Some(1),
            ..Default::default()
        };

        let response = exa.find_similar(request).await?;

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, "test_id");
        assert_eq!(response.results[0].title, "Test Title");
        assert_eq!(response.results[0].url, "https://example.com");
        assert_eq!(response.results[0].score, 0.95);
        assert_eq!(
            response.results[0].published_date,
            Some("2023-01-01".to_string())
        );
        assert_eq!(response.results[0].author, Some("Test Author".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_similar_error() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/findSimilar")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "code": "bad_request",
                    "message": "Invalid request parameters"
                })
                .to_string(),
            )
            .create();

        let exa = Exa::builder()
            .api_key("test_key".to_string())
            .base_url(mock_url)
            .build()?;

        let request = FindSimilarRequest {
            url: "".to_string(),
            ..Default::default()
        };

        let result = exa.find_similar(request).await;

        assert!(result.is_err());
        if let Err(ExaError::HttpError(error)) = result {
            assert_eq!(error.status, 400);
            assert_eq!(error.payload.code, "bad_request");
            assert_eq!(error.payload.message, "Invalid request parameters");
        } else {
            panic!("Expected HttpError");
        }

        Ok(())
    }

    #[test]
    fn test_find_similar_request_new_valid_url() {
        let url = "https://example.com";
        let request = FindSimilarRequest::new(url.to_string());
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.url, url);
    }

    #[test]
    fn test_find_similar_request_new_invalid_url() {
        let url = "not a valid url";
        let request = FindSimilarRequest::new(url.to_string());
        assert!(request.is_err());
    }
}
