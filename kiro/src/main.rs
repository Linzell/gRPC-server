// main.rs
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

#![recursion_limit = "256"]
#![allow(clippy::enum_variant_names)]
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use dotenv::dotenv;
use kiro_database::db_bridge::Database;
use rustls::crypto;

#[cfg(feature = "surrealdb")]
use kiro_database::SurrealDBRepo;

#[cfg(not(any(feature = "surrealdb", feature = "postgres")))]
use kiro_database::db_bridge::MockDatabaseOperations;

mod config;
mod error;
mod middleware;
mod prelude;
mod server;
mod utils;

use crate::{config::Config, server::Server, utils::telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load configuration
    let config = Config::init()?;

    if crypto::CryptoProvider::install_default(crypto::aws_lc_rs::default_provider()).is_err() {
        tracing::warn!("Failed to install default crypto provider");
    }

    if config.app.enable_tracing {
        #[cfg(feature = "tracing")]
        telemetry::init_tracer()?;
    }

    let addr = SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), config.ports.https());
    let server = Server::new(addr, config.clone()).await?;

    // Database setup
    #[cfg(feature = "surrealdb")]
    let db = {
        let surrealdb = SurrealDBRepo::init().await?;
        tracing::info!("🔧 Migrating database");
        SurrealDBRepo::migrate(surrealdb.db.clone()).await?;
        Database::Surreal(surrealdb.db)
    };

    #[cfg(not(any(feature = "surrealdb", feature = "postgres")))]
    let db = Database::Mock(MockDatabaseOperations::new());

    // Initialize services
    // await_initialization(&db).await?;

    // Start the server
    server.run(db, config).await?;

    #[cfg(feature = "tracing")]
    {
        opentelemetry::global::shutdown_tracer_provider();
        tracing::trace!("Stop tracing... 🛑");
    }

    Ok(())
}
