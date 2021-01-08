use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_yaml::Error),
}
