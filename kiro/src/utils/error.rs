use crate::config;

/// Error type for the API
///
/// ## Fields
///
/// - `AnyhowError` - anyhow::Error
/// - `IO` - std::io::Error
/// - `TomlDeError` - toml::de::Error
/// - `TomlSerError` - toml::ser::Error
/// - `Configuration` - config::ConfigError
/// - `TraceError` - opentelemetry::trace::TraceError
/// - `MetricsError` - opentelemetry::metrics::MetricsError
/// - `TryInitError` - tracing_subscriber::util::TryInitError
#[derive(thiserror::Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
    #[error("{0}")]
    AnyhowError(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    TomlDeError(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerError(#[from] toml::ser::Error),

    #[error(transparent)]
    Configuration(#[from] config::ConfigError),

    #[error(transparent)]
    TraceError(#[from] opentelemetry::trace::TraceError),

    #[error(transparent)]
    MetricsError(#[from] opentelemetry::metrics::MetricsError),

    #[error(transparent)]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::AnyhowError(err.to_string())
    }
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
            Error::TomlSerError(e) => tonic::Status::internal(e.to_string()),
            Error::Configuration(e) => tonic::Status::internal(e.to_string()),
            Error::TraceError(e) => tonic::Status::internal(e.to_string()),
            Error::MetricsError(e) => tonic::Status::internal(e.to_string()),
            Error::TryInitError(e) => tonic::Status::internal(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_anyhow() {
        let err = Error::AnyhowError("anyhow error".to_string());
        let status = tonic::Status::internal("anyhow error");
        assert_eq!(tonic::Status::from(err).message(), status.message());
    }

    #[test]
    fn test_error_io() {
        let err = Error::IO(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
        let status = tonic::Status::internal("io error");
        assert_eq!(tonic::Status::from(err).message(), status.message());
    }

    // #[test]
    // fn test_error_toml_de() {
    //     let err = Error::TomlDeError(toml::de::Error::from("toml error"));
    //     let status = tonic::Status::internal("toml error");
    //     assert_eq!(tonic::Status::from(err).message(), status.message());
    // }

    // #[test]
    // fn test_error_toml_ser() {
    //     let err = Error::TomlSerError(toml::ser::Error::custom("toml ser error"));
    //     let status = tonic::Status::internal("toml ser error");
    //     assert_eq!(tonic::Status::from(err).message(), status.message());
    // }

    #[test]
    fn test_error_configuration() {
        let err = Error::Configuration(config::ConfigError::new("config error".to_string()));
        let status = tonic::Status::internal("Configuration error: config error");
        assert_eq!(tonic::Status::from(err).message(), status.message());
    }

    #[test]
    fn test_error_trace() {
        let err = Error::TraceError(opentelemetry::trace::TraceError::Other(
            "trace error".into(),
        ));
        let status = tonic::Status::internal("trace error");
        assert_eq!(tonic::Status::from(err).message(), status.message());
    }

    #[test]
    fn test_error_metrics() {
        let err = Error::MetricsError(opentelemetry::metrics::MetricsError::Other(
            "metrics error".into(),
        ));
        let status = tonic::Status::internal("Metrics error: metrics error");
        assert_eq!(tonic::Status::from(err).message(), status.message());
    }

    // #[test]
    // fn test_error_try_init() {
    //     let err = Error::TryInitError(tracing_subscriber::util::TryInitError { _priv: () });
    //     let status =
    //         tonic::Status::internal("an error occurred when initializing the global subscriber");
    //     assert_eq!(tonic::Status::from(err).message(), status.message());
    // }
}
