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
use std::io;
use tonic::Status;

#[cfg(feature = "aws")]
use aws_sdk_s3::operation::{
    delete_object::DeleteObjectError, get_object::GetObjectError, put_object::PutObjectError,
};
#[cfg(feature = "aws")]
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Object not found in bucket")]
    NotFound,

    #[cfg(feature = "aws")]
    #[error("Failed to get object from S3: {0}")]
    S3GetError(#[from] SdkError<GetObjectError, Response>),

    #[cfg(feature = "aws")]
    #[error("Failed to put object to S3: {0}")]
    S3PutError(#[from] SdkError<PutObjectError, Response>),

    #[cfg(feature = "aws")]
    #[error("Failed to delete object from S3: {0}")]
    S3DeleteError(#[from] SdkError<DeleteObjectError, Response>),

    #[cfg(feature = "aws")]
    #[error("S3 error: {0}")]
    S3Error(String),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Invalid bucket configuration")]
    InvalidConfig,

    #[error("Failed to initialize client")]
    ClientInitError,

    #[error("Invalid object key format")]
    InvalidKeyFormat,

    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<StorageError> for DatabaseError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::Database(e) => e,
            _ => DatabaseError::Internal(error.to_string()),
        }
    }
}

impl From<StorageError> for Status {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::NotFound => Status::not_found("Object not found in bucket"),
            #[cfg(feature = "aws")]
            StorageError::S3GetError(e) => Status::internal(format!("S3 get error: {}", e)),
            #[cfg(feature = "aws")]
            StorageError::S3PutError(e) => Status::internal(format!("S3 put error: {}", e)),
            #[cfg(feature = "aws")]
            StorageError::S3DeleteError(e) => Status::internal(format!("S3 delete error: {}", e)),
            #[cfg(feature = "aws")]
            StorageError::S3Error(e) => Status::internal(format!("S3 error: {}", e)),
            StorageError::IO(e) => Status::internal(format!("IO error: {}", e)),
            StorageError::InvalidConfig => {
                Status::failed_precondition("Invalid bucket configuration")
            }
            StorageError::ClientInitError => Status::internal("Failed to initialize client"),
            StorageError::InvalidKeyFormat => Status::invalid_argument("Invalid object key format"),
            StorageError::Database(e) => e.into(),
            StorageError::Internal(msg) => Status::internal(msg),
        }
    }
}
