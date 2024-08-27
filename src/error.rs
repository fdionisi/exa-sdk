use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum ExaError {
    #[error("Client error: {0}")]
    ClientError(#[from] reqwest::Error),
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
    pub code: String,
    pub message: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {} - {}",
            self.status, self.payload.code, self.payload.message
        )
    }
}
