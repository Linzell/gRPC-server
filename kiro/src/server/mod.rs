// src/server/mod.rs
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

use axum_server::{tls_rustls::RustlsConfig, Handle};
use kiro_database::db_bridge::Database;
use std::net::SocketAddr;

mod certificate;
#[cfg(feature = "documentation")]
mod docs;
mod health;
mod redirect;
mod reflection;
mod setup;
mod shutdown;

pub use redirect::*;
pub use shutdown::*;

pub struct Server {
    addr: SocketAddr,
    tls_config: RustlsConfig,
    handle: Handle,
    config: crate::config::Config,
}

impl Server {
    pub async fn new(
        addr: SocketAddr, config: crate::config::Config,
    ) -> Result<Self, crate::error::ServerError> {
        Ok(Self {
            addr,
            tls_config: setup::create_tls_config(&config.certificate).await?,
            handle: Handle::new(),
            config,
        })
    }

    pub async fn run(
        self, db: Database, config: crate::config::Config,
    ) -> Result<(), crate::error::ServerError> {
        let app = setup::create_app(db, config.clone()).await?;

        // Create shutdown future
        let shutdown_future = shutdown_signal(self.handle.clone(), config);

        // Spawn HTTP redirect server
        tokio::spawn(redirect_http_to_https(self.config.ports, shutdown_future));

        axum_server::bind_rustls(self.addr, self.tls_config)
            .handle(self.handle.clone())
            .serve(app.into_make_service())
            .await
            .map_err(|e| crate::error::ServerError::ServerStartup(e.to_string()))
    }
}
