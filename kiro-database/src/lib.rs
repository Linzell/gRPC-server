// lib.rs
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

mod database;
mod error;
mod utils;

/// # Surreal Module
///
/// The surreal module provides surreal functionality.
#[cfg(feature = "surrealdb")]
pub use database::surrealdb_repo::SurrealDBRepo;

/// # Postgres Module
///
/// The postgres module provides postgres functionality.
#[cfg(feature = "postgres")]
pub use database::postgres_repo::PostgresRepo;

/// # Database Module
///
/// The database module provides database functionality.
pub use database::db_bridge;

/// # Error Module
///
/// The error module provides error handling functionality.
pub use error::DatabaseError;

/// Re-export database types and identifiers
///
/// The `DbId` type is a database-agnostic identifier that can represent various types
/// of database IDs (strings, numbers, arrays, objects). Along with its companion
///
/// The `DbDateTime` type provides timestamp functionality for database records.
pub use database::{DbDateTime, DbId, DbIdentifier};

/// # Utils Module
///
/// The utils module provides utility functions for the database.
pub use utils::{env::get_env_or, env::get_env_unsafe, env::get_envv};
