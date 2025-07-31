use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DarwinV7Error {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    CsvError(#[from] csv_async::Error),

    #[error(transparent)]
    SerdePathToError(#[from] serde_path_to_error::Error<serde_json::Error>),

    #[error("HTTP Error: {0} - {1}")]
    HTTPError(StatusCode, String),

    #[error("Missing value: {0}")]
    MissingValueError(String),

    #[error("Invalid config: {0}")]
    InvalidConfigError(String),

    #[error("Invalid annotation type: {0}")]
    InvalidAnnotationTypeError(String),

    #[error("Invalid dataset item status: {0}")]
    InvalidDatasetItemStatusError(String),

    #[error("Unable to find annotation class with name: {0}")]
    AnnotationClassNotFoundError(String),
}
