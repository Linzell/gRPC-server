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
pub enum SessionError {
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

    #[cfg(feature = "mailer")]
    #[error(transparent)]
    Mailer(#[from] lettre::error::Error),

    #[cfg(feature = "mailer")]
    #[error(transparent)]
    SMTP(#[from] lettre::transport::smtp::Error),

    #[cfg(feature = "mailer")]
    #[error(transparent)]
    MailerError(#[from] kiro_mailer::MailerError),

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl From<SessionError> for DatabaseError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::Database(e) => e,
            _ => DatabaseError::Internal(error.to_string()),
        }
    }
}

impl From<SessionError> for Status {
    fn from(error: SessionError) -> Self {
        match error {
            // Session errors
            SessionError::KeyGenerationFailed => Status::internal("Session key generation failed"),
            SessionError::NotCreated => Status::internal("Session not created"),
            SessionError::Expired => Status::unauthenticated("Session expired"),
            SessionError::NotFound => Status::unauthenticated("Session not found"),
            SessionError::General => Status::internal("Session error"),
            SessionError::RenewalFailed => Status::internal("Failed to renew session"),
            SessionError::DeletionFailed => Status::internal("Failed to delete session"),
            SessionError::DestroyAllFailed => Status::internal("Failed to destroy all sessions"),
            SessionError::IpMismatch => Status::permission_denied("Session IP address mismatch"),
            SessionError::PasswordHashingFailed => Status::internal("Password hashing failed"),
            SessionError::PasswordIncorrect => Status::unauthenticated("Password incorrect"),
            // Mailer errors
            SessionError::NewConnectionEmailFailed => {
                Status::internal("Failed to send new connection email")
            }
            SessionError::InvalidAddress(e) => Status::invalid_argument(e),
            #[cfg(feature = "mailer")]
            SessionError::Mailer(e) => Status::internal(e.to_string()),
            #[cfg(feature = "mailer")]
            SessionError::SMTP(e) => Status::internal(e.to_string()),
            #[cfg(feature = "mailer")]
            SessionError::MailerError(e) => e.into(),
            // Database errors
            SessionError::Database(e) => e.into(),
        }
    }
}
