// database/surrealdb_repo.rs
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

use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::MigrationRunner;

use crate::{database::db_types::DbId, error::DatabaseError, utils::env::get_env_or};

/// # SurrealDBRepo
///
/// The `SurrealDBRepo` struct is a struct that represents a repository for the SurrealDB database.
///
/// ```rust
/// #[derive(Clone)]
/// pub struct SurrealDBRepo {
///     pub db: Surreal<Any>,
/// }
/// ```
///
/// ## Methods
///
/// ### Init
///
/// The `init` method initializes the SurrealDB connection.
///
/// ```rust
/// let db = SurrealDBRepo::init().await?;
///
/// println!("🦋 Database: {:?}", db);
/// ```
///
/// ### Migrate
///
/// The `migrate` method migrates the database.
///
/// ```rust
/// SurrealDBRepo::migrate().await?;
///
/// println!("🦋 Database migrated");
/// ```
#[derive(Clone)]
pub struct SurrealDBRepo {
    pub db: Surreal<Any>,
}

/// # Record
///
/// The `Record` struct is a struct that represents a record in the SurrealDB database.
///
/// ```rust
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// pub struct Record<T> {
///     id: DbId,
///     #[serde(flatten)]
///     data: T,
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Record<T> {
    id: DbId,
    #[serde(flatten)]
    data: T,
}

impl SurrealDBRepo {
    /// # Init
    ///
    /// The `init` method initializes the SurrealDB connection.
    ///
    /// ```rust
    /// let db = SurrealDBRepo::init().await?;
    ///
    /// println!("🦋 Database: {:?}", db);
    /// ```
    pub async fn init() -> Result<Self, DatabaseError> {
        #[cfg(feature = "tracing")]
        tracing::info!("🦋 Connecting database...");

        let db_location = get_env_or("SURREAL_DB_HOST", "ws://127.0.0.1:8000");

        let db = any::connect(db_location.clone()).await.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("🔥 Failed to connect to the database: {}", e),
            )
        })?;

        let db_user = get_env_or("SURREAL_USER", "root");
        let db_pass = get_env_or("SURREAL_PASS", "root");

        #[cfg(feature = "tracing")]
        tracing::info!("🔐 Signing in...");

        db.signin(Root {
            username: db_user.as_str(),
            password: db_pass.as_str(),
        })
        .await
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("🔥 Failed to sign in: {}", e),
            )
        })?;

        let db_namespace = get_env_or("SURREAL_NAMESPACE", "test");
        let db_database = get_env_or("SURREAL_DATABASE", "test");

        db.use_ns(db_namespace.clone())
            .use_db(db_database.clone())
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("🔥 Failed to use namespace and database: {}", e),
                )
            })?;

        let session_db = any::connect(db_location).await.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("🔥 Failed to connect to the session database: {}", e),
            )
        })?;

        session_db
            .signin(Root {
                username: db_user.as_str(),
                password: db_pass.as_str(),
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("🔥 Failed to sign in for this session: {}", e),
                )
            })?;

        session_db
            .use_ns(db_namespace)
            .use_db(db_database)
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "🔥 Failed to use namespace and database for this session: {}",
                        e
                    ),
                )
            })?;

        Ok(Self { db })
    }

    /// # Migrate
    ///
    /// The `migrate` method migrates the database.
    ///
    /// ```rust
    /// SurrealDBRepo::migrate().await?;
    ///
    /// println!("🦋 Database migrated");
    /// ```
    pub async fn migrate(db: Surreal<Any>) -> Result<(), DatabaseError> {
        #[cfg(feature = "tracing")]
        tracing::info!("📦 Initializing the tables if necessary...");

        MigrationRunner::new(&db).up().await.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("🔥 Failed to migrate the database: {}", e),
            )
        })?;

        #[cfg(feature = "tracing")]
        tracing::info!("🎉 Database is ready");

        Ok(())
    }
}
