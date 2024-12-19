// database/mod.rs
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

mod db_types;

/// # SurrealDB Repository
///
/// The SurrealDB Repository module is a module that provides utilities for the SurrealDB repository.
#[cfg(feature = "surrealdb")]
pub mod surrealdb_repo;

// /// # Postgres Repository
// ///
// /// The Postgres Repository module is a module that provides utilities for the Postgres repository.
// #[cfg(feature = "postgres")]
// pub mod postgres_repo;

/// # Database Bridge
///
/// The database bridge module is a module that provides utilities for the database bridge.
pub mod db_bridge;

/// Re-export database types and identifiers
///
/// The `DbId` type is a database-agnostic identifier that can represent various types
/// of database IDs (strings, numbers, arrays, objects). Along with its companion
///
/// The `DbDateTime` type provides timestamp functionality for database records.
pub use db_types::{DbDateTime, DbId, DbIdentifier};
