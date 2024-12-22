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
pub enum MailerError {
    #[error("Link not found")]
    NotFound,

    #[error("Link expired")]
    Expired,

    #[error("Link creation failed")]
    CreationFailed,

    #[error("Link deletion failed")]
    DeletionFailed,

    #[error("Invalid link type")]
    InvalidType,

    #[error("Invalid link format")]
    InvalidFormat,

    #[error("Link already exists")]
    AlreadyExists,

    #[error("Failed to validate link")]
    ValidationFailed,

    #[error("{0} is not a recipient")]
    InvalidAddress(String),

    #[error(transparent)]
    Mailer(#[from] lettre::error::Error),

    #[error(transparent)]
    SMTP(#[from] lettre::transport::smtp::Error),

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl From<MailerError> for DatabaseError {
    fn from(error: MailerError) -> Self {
        match error {
            MailerError::Database(e) => e,
            _ => DatabaseError::Internal(error.to_string()),
        }
    }
}

impl From<MailerError> for Status {
    fn from(error: MailerError) -> Self {
        match error {
            MailerError::NotFound => Status::not_found("Link not found"),
            MailerError::Expired => Status::failed_precondition("Link has expired"),
            MailerError::CreationFailed => Status::internal("Failed to create link"),
            MailerError::DeletionFailed => Status::internal("Failed to delete link"),
            MailerError::InvalidType => Status::invalid_argument("Invalid link type"),
            MailerError::InvalidFormat => Status::invalid_argument("Invalid link format"),
            MailerError::AlreadyExists => Status::already_exists("Link already exists"),
            MailerError::ValidationFailed => Status::invalid_argument("Link validation failed"),
            MailerError::InvalidAddress(e) => {
                Status::invalid_argument(format!("{} is not a recipient", e))
            }
            MailerError::Mailer(e) => Status::internal(e.to_string()),
            MailerError::SMTP(e) => Status::internal(e.to_string()),
            MailerError::Database(e) => e.into(),
        }
    }
}
