// server/certificate.rs
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

use axum_server::tls_rustls::RustlsConfig;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use crate::config::CertificateConfig;

pub struct CertificateManager {
    config: CertificateConfig,
    work_dir: PathBuf,
}

impl CertificateManager {
    pub fn new(config: CertificateConfig) -> Self {
        Self {
            config,
            work_dir: PathBuf::from("certs"),
        }
    }

    pub async fn setup(&self) -> Result<RustlsConfig, std::io::Error> {
        // Ensure the certs directory exists
        self.ensure_certs_directory().await?;

        // Check if CA exists, if not generate it
        if !self.ca_exists().await {
            self.generate_ca().await?;
        }

        // Check if certificates exist and are valid
        if self.certificates_exist().await && self.verify_certificates().await? {
            return self.load_certificates().await;
        }

        // Generate new certificates signed by our CA
        self.generate_certificates().await?;
        self.load_certificates().await
    }

    async fn ensure_certs_directory(&self) -> Result<(), std::io::Error> {
        if !self.work_dir.exists() {
            fs::create_dir_all(&self.work_dir).await?;

            #[cfg(feature = "tracing")]
            tracing::info!("📁 Created certificates directory");
        }
        Ok(())
    }

    async fn ca_exists(&self) -> bool {
        Path::new(&self.config.ca_path).exists()
    }

    async fn certificates_exist(&self) -> bool {
        Path::new(&self.config.cert_path).exists() && Path::new(&self.config.key_path).exists()
    }

    async fn generate_ca(&self) -> Result<(), std::io::Error> {
        // Generate CA private key
        let ca_key_path = self.work_dir.join("ca.key");
        let output = Command::new("openssl")
            .args(&["genrsa", "-out", ca_key_path.to_str().unwrap(), "4096"])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to generate CA key: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        // Generate CA certificate
        let output = Command::new("openssl")
            .args(&[
                "req",
                "-x509",
                "-new",
                "-nodes",
                "-key",
                ca_key_path.to_str().unwrap(),
                "-sha256",
                "-days",
                "1825", // 5 years for CA
                "-out",
                self.config.ca_path.to_str().unwrap(),
                "-subj",
                &format!(
                    "/C={}/ST={}/L={}/O={} CA/OU={}/CN={} Root CA",
                    self.config.country,
                    self.config.state,
                    self.config.locality,
                    self.config.organization,
                    self.config.organizational_unit,
                    self.config.common_name
                ),
                "-addext",
                "basicConstraints=critical,CA:TRUE",
                "-addext",
                "keyUsage=critical,digitalSignature,keyCertSign,cRLSign",
            ])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to generate CA certificate: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        #[cfg(feature = "tracing")]
        tracing::info!("🔐 Generated CA certificate");

        Ok(())
    }

    async fn generate_certificates(&self) -> Result<(), std::io::Error> {
        // Generate private key
        let output = Command::new("openssl")
            .args(&[
                "genrsa",
                "-out",
                self.config.key_path.to_str().unwrap(),
                "2048",
            ])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to generate private key: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        // Generate CSR (Certificate Signing Request)
        let csr_path = self.work_dir.join("server.csr");
        let output = Command::new("openssl")
            .args(&[
                "req",
                "-new",
                "-key",
                self.config.key_path.to_str().unwrap(),
                "-out",
                csr_path.to_str().unwrap(),
                "-subj",
                &self.config.subject(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to generate CSR: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        // Create extension file for SAN
        let ext_path = self.work_dir.join("server.ext");
        let ext_content = format!(
            "authorityKeyIdentifier=keyid,issuer\n\
             basicConstraints=CA:FALSE\n\
             keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment\n\
             subjectAltName = DNS:{},DNS:*.{}\n",
            self.config.common_name, self.config.common_name
        );
        fs::write(&ext_path, ext_content).await?;

        // Sign the certificate with our CA
        let output = Command::new("openssl")
            .args(&[
                "x509",
                "-req",
                "-in",
                csr_path.to_str().unwrap(),
                "-CA",
                self.config.ca_path.to_str().unwrap(),
                "-CAkey",
                self.work_dir.join("ca.key").to_str().unwrap(),
                "-CAcreateserial",
                "-out",
                self.config.cert_path.to_str().unwrap(),
                "-days",
                &self.config.days_valid.to_string(),
                "-sha256",
                "-extfile",
                ext_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to sign certificate: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        // Clean up temporary files
        let _ = fs::remove_file(csr_path).await;
        let _ = fs::remove_file(ext_path).await;

        #[cfg(feature = "tracing")]
        tracing::info!("📜 Generated and signed server certificates");

        Ok(())
    }

    async fn load_certificates(&self) -> Result<RustlsConfig, std::io::Error> {
        let cert = fs::read(&self.config.cert_path).await?;
        let key = fs::read(&self.config.key_path).await?;

        RustlsConfig::from_pem(cert, key).await.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("TLS error: {}", e))
        })
    }

    async fn verify_certificates(&self) -> Result<bool, std::io::Error> {
        let output = Command::new("openssl")
            .args(&[
                "verify",
                "-CAfile",
                self.config.ca_path.to_str().unwrap(),
                "-verify_hostname",
                &self.config.common_name,
                self.config.cert_path.to_str().unwrap(),
            ])
            .output()?;

        Ok(output.status.success())
    }
}
