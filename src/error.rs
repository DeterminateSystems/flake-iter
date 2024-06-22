#[derive(Debug, thiserror::Error)]
pub enum FlakeIterError {
    #[error("environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Misc(String),

    #[error("error converting UTF-8 to string: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
