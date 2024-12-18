// error.rs
//
// Copyright Charlie Cohen <linzellart@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::num::ParseIntError;
use tonic::Status;

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
/// - `SurrealDBError` - surrealdb::Error
#[derive(thiserror::Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum ServerError {
    #[error("{0}")]
    AnyhowError(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    TraceError(#[from] opentelemetry::trace::TraceError),

    #[error(transparent)]
    MetricsError(#[from] opentelemetry::metrics::MetricsError),

    #[error(transparent)]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),

    #[error("Server startup failed: {0}")]
    ServerStartup(String),

    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] ParseIntError),
}

/// Convert Error to tonic::Status
///
/// ## Arguments
///
/// - `error` - Error to convert
impl From<ServerError> for tonic::Status {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::AnyhowError(e) => tonic::Status::internal(e.to_string()),
            ServerError::IO(e) => tonic::Status::internal(e.to_string()),
            ServerError::TraceError(e) => Status::internal(e.to_string()),
            ServerError::MetricsError(e) => Status::internal(e.to_string()),
            ServerError::TryInitError(e) => Status::internal(e.to_string()),
            ServerError::ServerStartup(e) => Status::internal(e),
            ServerError::ParseInt(e) => Status::invalid_argument(e.to_string()),
        }
    }
}
