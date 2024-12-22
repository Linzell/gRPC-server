// error.rs
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

use tonic::Status;

#[derive(thiserror::Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum DatabaseError {
    #[error("Database Record that was just checked doesn't exist?")]
    DBOptionNone,

    // Database errors
    #[error("Surreal initialization failed")]
    DBConnectionError,

    #[cfg(feature = "surrealdb")]
    #[error("SurrealDB error: {0}")]
    SurrealDB(#[from] surrealdb::Error),

    #[cfg(feature = "surrealdb")]
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert Error to tonic::Status
///
/// ## Arguments
///
/// - `error` - Error to convert
impl From<DatabaseError> for Status {
    fn from(error: DatabaseError) -> Self {
        match error {
            // Database errors
            DatabaseError::DBOptionNone => {
                Status::internal("Database Record that was just checked doesn't exist?")
            }
            #[cfg(feature = "surrealdb")]
            DatabaseError::SurrealDB(e) => Status::internal(e.to_string()),
            DatabaseError::DBConnectionError => Status::internal("Surreal initialization failed"),
            // Token errors
            #[cfg(feature = "surrealdb")]
            DatabaseError::Io(e) => Status::internal(e.to_string()),
            DatabaseError::SerdeJson(e) => Status::internal(e.to_string()),
            // DK errors
            DatabaseError::Internal(e) => Status::internal(e),
        }
    }
}
