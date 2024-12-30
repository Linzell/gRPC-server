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
/// Repository implementation for SurrealDB database connectivity and operations.
///
/// ## Features
/// - Manages database connection and authentication
/// - Handles database migrations
/// - Provides transaction support
/// - Support for multiple database namespaces
///
/// ## Examples
/// ```rust,no_run
/// use db::SurrealDBRepo;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize database connection
///     let repo = SurrealDBRepo::init().await?;
///
///     // Migrate database schema
///     SurrealDBRepo::migrate(repo.db.clone()).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct SurrealDBRepo {
    pub db: Surreal<Any>,
}

/// # Record
///
/// Generic record type for SurrealDB stored data.
///
/// ## Type Parameters
/// - `T`: The wrapped data type for the record
///
/// ## Fields
/// - `id`: Unique record identifier
/// - `data`: The actual record data
///
/// ## Examples
/// ```rust
/// use db::{Record, User};
///
/// let user = User { name: "Alice".into() };
/// let record = Record {
///     id: DbId::new("user"),
///     data: user
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Record<T> {
    id: DbId,
    #[serde(flatten)]
    data: T,
}

impl SurrealDBRepo {
    /// Initializes a new SurrealDB connection with authentication
    ///
    /// # Environment Variables
    /// - `SURREAL_DB_HOST`: Database host URL (default: ws://127.0.0.1:8000)
    /// - `SURREAL_USER`: Database username (default: root)
    /// - `SURREAL_PASS`: Database password (default: root)
    /// - `SURREAL_NAMESPACE`: Database namespace (default: test)
    /// - `SURREAL_DATABASE`: Database name (default: test)
    ///
    /// # Returns
    /// - `Result<Self, DatabaseError>`: Connected repository instance
    ///
    /// # Examples
    /// ```rust,no_run
    /// let repo = SurrealDBRepo::init().await?;
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

    /// Runs database migrations to update schema
    ///
    /// # Arguments
    /// - `db`: Database connection instance
    ///
    /// # Returns
    /// - `Result<(), DatabaseError>`: Success or error
    ///
    /// # Examples
    /// ```rust,no_run
    /// SurrealDBRepo::migrate(db).await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_init_connection() {
        let result = SurrealDBRepo::init().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_migrations() {
        let repo = SurrealDBRepo::init().await.unwrap();
        let result = SurrealDBRepo::migrate(repo.db).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_record_serialization() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            value: String,
        }

        let record = Record {
            id: DbId::default(),
            data: TestData {
                value: "default".into(),
            },
        };

        let serialized = serde_json::to_string(&record).unwrap();
        let deserialized: Record<TestData> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(record, deserialized);
    }
}
