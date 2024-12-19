// config/app.rs
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

use kiro_database::get_env_or;

use crate::error::ServerError;

/// Application-wide configuration settings
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub frontend_url: String,
    pub environment: Environment,
    pub enable_tracing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Testing,
}

impl AppConfig {
    pub(crate) fn init() -> Result<Self, ServerError> {
        Ok(Self {
            frontend_url: get_env_or("FRONT_CONNECT_URL", "http://localhost:5173"),
            environment: Environment::from_env(),
            enable_tracing: cfg!(feature = "tracing"),
        })
    }
}

impl Environment {
    fn from_env() -> Self {
        match get_env_or("ENVIRONMENT", "development").as_str() {
            "production" => Self::Production,
            "testing" => Self::Testing,
            _ => Self::Development,
        }
    }

    #[cfg(feature = "mailer")]
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }
}
