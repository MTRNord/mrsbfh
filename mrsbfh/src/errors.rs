//! # Errors that the helpers can return

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_yaml::Error),
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ExtensionRejection {
    #[error("Extensions taken by other extractor")]
    ExtensionsAlreadyExtracted,
    #[error("{0}")]
    MissingExtension(String),
}

#[derive(Error, Debug)]
pub enum Errors {
    #[error("{0}")]
    CustomError(String),
}
