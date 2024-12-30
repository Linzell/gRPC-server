// config/mod.rs
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

mod app;
mod certificate;
mod logging_config;
mod ports;

pub use app::AppConfig;
#[cfg(feature = "mailer")]
pub use app::Environment;
pub use certificate::CertificateConfig;
pub use logging_config::{ErrorContext, ErrorSeverity, LoggingConfig};
pub use ports::Ports;

use crate::error::ServerError;

/// Configuration for the entire application
#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub ports: Ports,
    pub certificate: CertificateConfig,
}

impl Config {
    /// Initialize configuration from environment variables
    pub fn init() -> Result<Self, ServerError> {
        Ok(Self {
            app: AppConfig::init()?,
            ports: Ports::init()?,
            certificate: CertificateConfig::init()?,
        })
    }
}
