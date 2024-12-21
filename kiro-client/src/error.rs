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

use kiro_database::DatabaseError;
use tonic::Status;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Database Record that was just checked doesn't exist?")]
    DBOptionNone,

    #[error("Failed to generate invitation code")]
    InvitationCodeGeneration,

    #[cfg(feature = "storage")]
    #[error(transparent)]
    StorageError(#[from] dk_storage::StorageError),

    #[cfg(feature = "email")]
    #[error(transparent)]
    MailerError(#[from] dk_mailer::MailerError),

    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error(transparent)]
    Tonic(#[from] tonic::Status),
}

impl From<ClientError> for DatabaseError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Database(e) => e,
            _ => DatabaseError::Internal(error.to_string()),
        }
    }
}

impl From<ClientError> for Status {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::DBOptionNone => {
                Status::not_found("Database Record that was just checked doesn't exist?")
            }
            ClientError::InvitationCodeGeneration => {
                Status::internal("Failed to generate invitation code")
            }
            ClientError::Database(e) => e.into(),
            #[cfg(feature = "storage")]
            ClientError::StorageError(e) => e.into(),
            #[cfg(feature = "email")]
            ClientError::MailerError(e) => e.into(),
            ClientError::Tonic(e) => e,
        }
    }
}
