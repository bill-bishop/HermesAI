use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
