use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum ExaError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Client error: {0}")]
    ClientError(#[from] http_client::http::Error),
    #[error("HTTP error: {0}")]
    HttpError(HttpError),
}

#[derive(Debug, serde::Deserialize, serde::Serialize, thiserror::Error)]
pub struct HttpError {
    pub status: u16,
    pub payload: HttpErrorPayload,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HttpErrorPayload {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub error: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {} - {}",
            self.status, self.payload.request_id, self.payload.error
        )
    }
}
