// config/certificate.rs
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
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CertificateConfig {
    pub country: String,
    pub state: String,
    pub locality: String,
    pub organization: String,
    pub organizational_unit: String,
    pub common_name: String,
    pub days_valid: u32,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_path: PathBuf,
}

impl CertificateConfig {
    pub fn init() -> Result<Self, crate::error::ServerError> {
        Ok(Self {
            country: get_env_or("CERT_COUNTRY", "US"),
            state: get_env_or("CERT_STATE", "California"),
            locality: get_env_or("CERT_LOCALITY", "San Francisco"),
            organization: get_env_or("CERT_ORGANIZATION", "Kiro"),
            organizational_unit: get_env_or("CERT_ORGANIZATIONAL_UNIT", "Engineering"),
            common_name: get_env_or("CERT_COMMON_NAME", "localhost"),
            days_valid: get_env_or("CERT_DAYS_VALID", "365").parse()?,
            cert_path: PathBuf::from(get_env_or("CERT_PATH", "certs/cert.pem")),
            key_path: PathBuf::from(get_env_or("CERT_KEY_PATH", "certs/key.pem")),
            ca_path: PathBuf::from(get_env_or("CERT_CA_PATH", "certs/ca.pem")),
        })
    }

    pub fn subject(&self) -> String {
        format!(
            "/C={}/ST={}/L={}/O={}/OU={}/CN={}",
            self.country,
            self.state,
            self.locality,
            self.organization,
            self.organizational_unit,
            self.common_name
        )
    }
}
