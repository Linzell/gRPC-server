/// Macro used to test Token functions
/// Macro that returns a `gRPC error` message
/// Can take a string literal as an argument to change the error message
///
/// ## Usage
/// ```no_run
/// tonic_auth!() // Returns a "Unauthenticated" message
/// tonic_auth!("Token authentication error") // Returns a "Token authentication error" message
/// ```
#[macro_export]
macro_rules! tonic_auth {
    ($result:expr, $message:expr) => {
        $result.map_err(|e| {
            tracing::log::error!("{}", e);
            let msg = format!("{} : {}", $message, e);
            tonic::Status::unauthenticated(msg)
        })?
    };
}

/// Macro used to return version
/// Macro that returns the version of the crate
/// Can take a string literal as an argument to change the version message
///
/// ## Usage
/// ```no_run
/// version!() // Returns the version of the crate
/// version!("Version: ") // Returns the version of the crate with a prefix
/// ```
#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
    ($prefix:expr) => {
        format!("{}{}", $prefix, env!("CARGO_PKG_VERSION"))
    };
}
