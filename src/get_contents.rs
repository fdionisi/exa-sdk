use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Exa, ExaError};

#[derive(Debug, Serialize)]
pub struct ContentsRequest {
    pub ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ContentsTextRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<ContentsHighlightsRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ContentsSummaryRequest>,
}

#[derive(Debug, Serialize)]
pub struct ContentsTextRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_html_tags: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ContentsHighlightsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_sentences: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights_per_url: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContentsSummaryRequest {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContentsResponse {
    pub results: Vec<ContentsResult>,
}

#[derive(Debug, Deserialize)]
pub struct ContentsResult {
    pub id: String,
    pub url: String,
    pub title: String,
    pub text: Option<String>,
    pub highlights: Option<Vec<String>>,
    pub highlight_scores: Option<Vec<f64>>,
}

impl Exa {
    pub async fn get_contents(
        &self,
        request: ContentsRequest,
    ) -> Result<ContentsResponse, ExaError> {
        self.post("/contents", request).await
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use mockito::Server as MockServer;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_get_contents() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/contents")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "results": [{
                        "id": "test_id",
                        "url": "https://example.com",
                        "title": "Test Title",
                        "text": "Test content",
                        "highlights": ["Test highlight"],
                        "highlight_scores": [0.95]
                    }]
                })
                .to_string(),
            )
            .create();

        let exa = Exa::builder()
            .api_key("test_key".to_string())
            .base_url(mock_url)
            .build()?;

        let request = ContentsRequest {
            ids: vec!["test_id".to_string()],
            text: Some(ContentsTextRequest {
                max_characters: Some(100),
                include_html_tags: Some(false),
            }),
            highlights: Some(ContentsHighlightsRequest {
                num_sentences: Some(1),
                highlights_per_url: Some(1),
                query: Some("test".to_string()),
            }),
            summary: None,
        };

        let response = exa.get_contents(request).await?;

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, "test_id");
        assert_eq!(response.results[0].url, "https://example.com");
        assert_eq!(response.results[0].title, "Test Title");
        assert_eq!(response.results[0].text, Some("Test content".to_string()));
        assert_eq!(
            response.results[0].highlights,
            Some(vec!["Test highlight".to_string()])
        );
        assert_eq!(response.results[0].highlight_scores, Some(vec![0.95]));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_contents_error() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/contents")
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

        let request = ContentsRequest {
            ids: vec![],
            text: None,
            highlights: None,
            summary: None,
        };

        let result = exa.get_contents(request).await;

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
}
