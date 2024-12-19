// config/ports.rs
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

/// Network port configuration
#[derive(Debug, Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

impl Ports {
    pub(crate) fn init() -> Result<Self, ServerError> {
        Ok(Self {
            http: get_env_or("HTTP_PORT", "3080").parse()?,
            https: get_env_or("HTTPS_PORT", "3000").parse()?,
        })
    }

    /// Get the HTTP port
    pub fn http(&self) -> u16 {
        self.http
    }

    /// Get the HTTPS port
    pub fn https(&self) -> u16 {
        self.https
    }
}
