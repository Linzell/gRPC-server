use crate::config;

/// Error type for the API
///
/// ## Fields
///
/// - `AnyhowError` - anyhow::Error
/// - `IO` - std::io::Error
/// - `TomlDeError` - toml::de::Error
#[derive(thiserror::Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    TomlDeError(#[from] toml::de::Error),

    #[error(transparent)]
    Configuration(#[from] config::ConfigError),
}

/// Convert Error to tonic::Status
///
/// ## Arguments
///
/// - `error` - Error to convert
impl From<Error> for tonic::Status {
    fn from(error: Error) -> Self {
        match error {
            Error::AnyhowError(e) => tonic::Status::internal(e.to_string()),
            Error::IO(e) => tonic::Status::internal(e.to_string()),
            Error::TomlDeError(e) => tonic::Status::internal(e.to_string()),
            Error::Configuration(e) => tonic::Status::internal(e.to_string()),
        }
    }
}
