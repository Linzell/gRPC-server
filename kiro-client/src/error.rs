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
    #[error("Session key generation failed")]
    KeyGenerationFailed,

    #[error("Session not created")]
    NotCreated,

    #[error("Session expired")]
    Expired,

    #[error("Session not found")]
    NotFound,

    #[error("Session error")]
    General,

    #[error("Failed to renew session")]
    RenewalFailed,

    #[error("Failed to delete session")]
    DeletionFailed,

    #[error("Failed to destroy all sessions")]
    DestroyAllFailed,

    #[error("Session IP address mismatch")]
    IpMismatch,

    #[error("Password hashing failed")]
    PasswordHashingFailed,

    #[error("Password incorrect")]
    PasswordIncorrect,

    #[error("Failed to send new connection email")]
    NewConnectionEmailFailed,

    #[error("{0} is not a recipient")]
    InvalidAddress(String),

    #[error("Encrypted data is invalid")]
    EncryptionError,

    #[error("Decrypted data is invalid")]
    DecryptionError,

    #[cfg(feature = "storage")]
    #[error(transparent)]
    StorageError(#[from] kiro_storage::StorageError),

    #[cfg(feature = "mailer")]
    #[error(transparent)]
    MailerError(#[from] kiro_mailer::MailerError),

    #[error("Database Record that was just checked doesn't exist?")]
    DBOptionNone,

    #[error(transparent)]
    Database(#[from] DatabaseError),
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
            // Session errors
            ClientError::KeyGenerationFailed => Status::internal("Session key generation failed"),
            ClientError::NotCreated => Status::internal("Session not created"),
            ClientError::Expired => Status::unauthenticated("Session expired"),
            ClientError::NotFound => Status::unauthenticated("Session not found"),
            ClientError::General => Status::internal("Session error"),
            ClientError::RenewalFailed => Status::internal("Failed to renew session"),
            ClientError::DeletionFailed => Status::internal("Failed to delete session"),
            ClientError::DestroyAllFailed => Status::internal("Failed to destroy all sessions"),
            ClientError::IpMismatch => Status::permission_denied("Session IP address mismatch"),
            ClientError::PasswordHashingFailed => Status::internal("Password hashing failed"),
            ClientError::PasswordIncorrect => Status::unauthenticated("Password incorrect"),
            // Mailer errors
            ClientError::NewConnectionEmailFailed => {
                Status::internal("Failed to send new connection email")
            }
            ClientError::InvalidAddress(e) => Status::invalid_argument(e),
            ClientError::EncryptionError => Status::internal("Encrypted data is invalid"),
            ClientError::DecryptionError => Status::internal("Decrypted data is invalid"),
            // Storage errors
            #[cfg(feature = "storage")]
            ClientError::StorageError(e) => e.into(),
            // Mailer errors
            #[cfg(feature = "mailer")]
            ClientError::MailerError(e) => e.into(),
            // Database errors
            ClientError::DBOptionNone => {
                Status::not_found("Database Record that was just checked doesn't exist?")
            }
            ClientError::Database(e) => e.into(),
        }
    }
}
