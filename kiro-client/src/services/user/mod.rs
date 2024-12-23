// services/user/mod.rs
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

//! User service implementation
//!
//! This module provides the core user functionality including:
//! - Reading user data
//! - Updating user data
//! - Deleting user data
//! - Disabling user accounts
//! - Sending emails to change email or password
//! - Uploading avatars
//! - Changing language, theme, notifications, privacy, and security settings
//!
//! The service is implemented as a gRPC service using the tonic framework.

use tonic::{async_trait, Request, Response, Status};

use kiro_api::{
    client::v1::{
        client_service_server::{self, ClientServiceServer},
        UpdateEmailRequest, UpdateLanguageRequest, UpdateNotificationsRequest,
        UpdatePasswordRequest, UpdatePrivacyRequest, UpdateSecurityRequest, UpdateThemeRequest,
        UploadAvatarRequest, UploadAvatarResponse,
    },
    google::protobuf::Empty,
};
use kiro_database::db_bridge::Database;

mod delete_user;
mod disable_user;
mod read_user;
mod send_email_to_change_email;
mod send_email_to_change_password;
mod update_email;
mod update_language;
mod update_notifications;
mod update_password;
mod update_privacy;
mod update_security;
mod update_theme;
mod upload_avatar;

/// The main authentication service implementation
#[derive(Clone)]
pub struct ClientService {
    /// Database connection for persistence
    pub db: Database,
}

impl ClientService {
    /// Creates a new instance of the client service
    ///
    /// # Arguments
    /// * `db` - Database connection to use for persistence
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Builds a new gRPC service server with this implementation
    ///
    /// # Arguments
    /// * `db` - Database connection to use for persistence
    ///
    /// # Returns
    /// A configured gRPC service ready to handle client requests
    pub fn build(db: Database) -> ClientServiceServer<Self> {
        ClientServiceServer::new(Self::new(db))
    }
}

#[async_trait]
impl client_service_server::ClientService for ClientService {
    type ReadUserStream = read_user::ReadUserStream;

    async fn read_user(
        &self, request: Request<Empty>,
    ) -> Result<Response<Self::ReadUserStream>, Status> {
        read_user::read_user(self, request).await
    }

    async fn delete_user(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        delete_user::delete_user(self, request).await
    }

    async fn disable_user(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        disable_user::disable_user(self, request).await
    }

    async fn send_email_to_change_email(
        &self, request: Request<Empty>,
    ) -> Result<Response<Empty>, Status> {
        #[cfg(not(feature = "mailer"))]
        Err(Status::unimplemented("Email functionality is disabled"));
        #[cfg(feature = "mailer")]
        send_email_to_change_email::send_email_to_change_email(self, request).await
    }

    async fn send_email_to_change_password(
        &self, request: Request<Empty>,
    ) -> Result<Response<Empty>, Status> {
        #[cfg(not(feature = "mailer"))]
        Err(Status::unimplemented("Email functionality is disabled"));
        #[cfg(feature = "mailer")]
        send_email_to_change_password::send_email_to_change_password(self, request).await
    }

    async fn update_email(
        &self, request: Request<UpdateEmailRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_email::update_email(self, request).await
    }

    async fn update_password(
        &self, request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_password::update_password(self, request).await
    }

    async fn upload_avatar(
        &self, request: Request<UploadAvatarRequest>,
    ) -> Result<Response<UploadAvatarResponse>, Status> {
        #[cfg(not(feature = "storage"))]
        Err(Status::unimplemented("Storage functionality is disabled"));
        #[cfg(feature = "storage")]
        upload_avatar::upload_avatar(self, request).await
    }

    async fn update_language(
        &self, request: Request<UpdateLanguageRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_language::update_language(self, request).await
    }

    async fn update_theme(
        &self, request: Request<UpdateThemeRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_theme::update_theme(self, request).await
    }

    async fn update_notifications(
        &self, request: Request<UpdateNotificationsRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_notifications::update_notifications(self, request).await
    }

    async fn update_privacy(
        &self, request: Request<UpdatePrivacyRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_privacy::update_privacy(self, request).await
    }

    async fn update_security(
        &self, request: Request<UpdateSecurityRequest>,
    ) -> Result<Response<Empty>, Status> {
        update_security::update_security(self, request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kiro_database::db_bridge::MockDatabaseOperations;

    #[tokio::test]
    async fn test_service_creation() {
        let mock_db = MockDatabaseOperations::new();
        let db = Database::Mock(mock_db);

        let service = ClientService::new(db);

        assert!(matches!(service.db, Database::Mock(_)));
    }

    #[tokio::test]
    async fn test_service_builder() {
        let mock_db = MockDatabaseOperations::new();
        let db = Database::Mock(mock_db);

        let service = ClientService::build(db);

        assert!(matches!(service, ClientServiceServer { .. }));
    }
}
