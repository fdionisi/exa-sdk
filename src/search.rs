use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::ExposeSecret;

use crate::{Exa, ExaError, HttpError, HttpErrorPayload};

impl Exa {
    /// Performs a search request to the Exa API.
    ///
    /// This method sends a POST request to the Exa API's search endpoint with the provided
    /// search parameters. It handles authentication, request construction, and response parsing.
    ///
    /// # Arguments
    ///
    /// * `request` - A `SearchRequest` struct containing the search parameters.
    ///
    /// # Returns
    ///
    /// Returns a `Result<SearchResponse, anyhow::Error>`. On success, it contains the parsed
    /// `SearchResponse`. On failure, it returns an error, which could be due to network issues,
    /// API errors, or parsing problems.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The API key is invalid or cannot be converted to a header value
    /// - The network request fails
    /// - The API returns a non-success status code
    /// - The response cannot be parsed into a `SearchResponse`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exa_sdk::{Exa, SearchRequest};
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// # let exa = Exa::builder().api_key("your_api_key".into()).build()?;
    /// let request = SearchRequest {
    ///     query: "Rust programming".to_string(),
    ///     num_results: Some(5),
    ///     ..Default::default()
    /// };
    ///
    /// let response = exa.search(request).await?;
    /// println!("Found {} results", response.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse, ExaError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&format!("Bearer {}", self.api_key.expose_secret()))
                .expect("couldn't create header value"),
        );

        let response = self
            .client
            .post(format!("{}/search", self.base_url))
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

        let search_response = response.json::<SearchResponse>().await?;
        Ok(search_response)
    }
}

/// Represents the response from a search request to the Exa API
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SearchResponse {
    /// List of search results
    pub results: Vec<SearchResult>,
    /// The Exa query created by the autoprompt functionality.
    #[serde(rename = "autopromptString")]
    pub autoprompt_string: Option<String>,
    /// If applicable, the date filter intelligently inferred from input queries that have autopropmpt on.
    #[serde(rename = "autoDate")]
    pub auto_date: Option<String>,
}

/// Represents a single search result from the Exa API
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,
    /// URL of the search result
    pub url: String,
    /// Publication date of the result, if available
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    /// Author of the result, if available
    pub author: Option<String>,
    /// Relevance score of the result
    pub score: f64,
    pub id: String,
    pub text: Option<String>,
    pub highlights: Option<Vec<String>>,
    #[serde(rename = "highlightScores")]
    pub highlight_scores: Option<Vec<f64>>,
}

/// Represents a search request to the Exa API
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SearchRequest {
    /// The search query string
    pub query: String,
    /// Whether to use autoprompt for query expansion
    #[serde(skip_serializing_if = "Option::is_none", rename = "useAutoprompt")]
    pub use_autoprompt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub kind: Option<SearchKind>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeText")]
    pub include_text: Option<Vec<String>>,
    /// Number of results to return (default: 10, max: 100)
    #[serde(skip_serializing_if = "Option::is_none", rename = "numResults")]
    pub num_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeDomains")]
    pub include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "excludeDomains")]
    pub exclude_domains: Option<Vec<String>>,
    /// Start date for crawled links (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none", rename = "startCrawlDate")]
    pub start_crawl_date: Option<String>,
    /// End date for crawled links (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none", rename = "endCrawlDate")]
    pub end_crawl_date: Option<String>,
    /// Start date for published links (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none", rename = "startPublishedDate")]
    pub start_published_date: Option<String>,
    /// End date for published links (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none", rename = "endPublishedDate")]
    pub end_published_date: Option<String>,
    /// Strings to exclude from webpage text (max 1 string, 5 words)
    #[serde(skip_serializing_if = "Option::is_none", rename = "excludeText")]
    pub exclude_text: Option<Vec<String>>,
    pub contents: Option<SearchContent>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SearchContent {
    pub text: Option<SearchContentText>,
    pub highlights: Option<SearchHighlights>,
    pub summary: Option<SearchSummary>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SearchContentText {
    /// Max length in characters for the text returned
    #[serde(rename = "maxCharacters")]
    pub max_characters: Option<u32>,
    /// Whether HTML tags, which can help the LLM understand structure of text, should be included. Default false
    #[serde(rename = "includeHtmlTags")]
    pub include_html_tags: Option<bool>,
}

/// Represents the highlights configuration for search results
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SearchHighlights {
    /// The number of sentences to be returned in each snippet. Default 5
    #[serde(rename = "numSentences")]
    pub num_sentences: Option<u32>,
    /// The number of snippets to return per page. Default 1
    #[serde(rename = "highlightsPerUrl")]
    pub highlights_per_url: Option<u32>,
    /// If specified, targets snippets most relevant to the query. In search, defaults to the search query.
    pub query: Option<String>,
}

/// Represents a summary of a webpage
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SearchSummary {
    /// Summary of the webpage
    pub summary: String,
    /// If specified, tries to answer the query in the summary
    #[serde(rename = "query")]
    pub query: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchKind {
    Neural,
    Keyword,
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use mockito::Server as MockServer;
    use serde_json::json;

    fn setup(base_url: String) -> Result<Exa> {
        Ok(Exa::builder()
            .api_key("test_key".to_string())
            .base_url(base_url)
            .build()?)
    }

    #[tokio::test]
    async fn test_basic_search() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "results": [{
                        "id": "Test ID",
                        "title": "Test Result",
                        "url": "https://example.com",
                        "score": 0.95
                    }],
                    "autopromptString": null
                })
                .to_string(),
            )
            .create();

        let exa = setup(mock_url)?;

        let request = SearchRequest {
            query: "test query".to_string(),
            use_autoprompt: None,
            num_results: None,
            ..Default::default()
        };

        let response = exa.search(request).await.unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Test Result");
        assert_eq!(response.results[0].url, "https://example.com");
        assert_eq!(response.results[0].score, 0.95);
        assert!(response.autoprompt_string.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_search_with_all_options() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "results": [{
                        "id": "Test ID",
                        "title": "Test Result",
                        "url": "https://example.com",
                        "score": 0.95,
                        "publishedDate": "2023-01-01"
                    }],
                    "autopromptString": "Expanded query"
                })
                .to_string(),
            )
            .create();

        let exa = setup(mock_url)?;

        let request = SearchRequest {
            query: "test query".to_string(),
            use_autoprompt: Some(true),
            num_results: Some(1),
            ..Default::default()
        };

        let response = exa.search(request).await.unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(
            response.autoprompt_string,
            Some("Expanded query".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_search_api_error() -> Result<()> {
        let mut server = MockServer::new_async().await;
        let mock_url = server.url();

        let _m = server
            .mock("POST", "/search")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "code": "unauthorized",
                    "message": "Your request was unauthorized"
                })
                .to_string(),
            )
            .create();

        let exa = setup(mock_url)?;

        let request = SearchRequest {
            query: "test query".to_string(),
            use_autoprompt: None,
            num_results: None,
            ..Default::default()
        };

        let result = exa.search(request).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "HTTP error: 401 - unauthorized - Your request was unauthorized"
        );
        Ok(())
    }
}
