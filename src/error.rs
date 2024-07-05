#[derive(Debug, thiserror::Error)]
pub(super) enum FlakeIterError {
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Misc(String),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}
