// services/auth/mod.rs
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

//! Authentication service implementation
//!
//! This module provides the core authentication functionality including:
//! - User registration
//! - Login/logout flows
//! - Session management
//!
//! The service is implemented as a gRPC service using the tonic framework.

use tonic::{async_trait, Request, Response, Status};

use kiro_api::{
    auth::v1::{
        auth_service_server::{self, AuthServiceServer},
        AuthRequest, Session,
    },
    google::protobuf::Empty,
};
use kiro_database::db_bridge::Database;

mod login;
mod logout;
mod register;

/// The main authentication service implementation
#[derive(Clone)]
pub struct AuthService {
    /// Database connection for persistence
    pub db: Database,
}

impl AuthService {
    /// Creates a new instance of the auth service
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
    /// A configured gRPC service ready to handle auth requests
    pub fn build(db: Database) -> AuthServiceServer<Self> {
        AuthServiceServer::new(Self::new(db))
    }
}

#[async_trait]
impl auth_service_server::AuthService for AuthService {
    /// Handles user login requests
    ///
    /// # Arguments
    /// * `request` - The login request containing credentials
    ///
    /// # Returns
    /// A new session if login is successful
    async fn login(&self, request: Request<AuthRequest>) -> Result<Response<Session>, Status> {
        login::login(self, request).await
    }

    /// Handles user logout requests
    ///
    /// # Arguments
    /// * `request` - Empty request to logout current session
    ///
    /// # Returns
    /// Empty response on successful logout
    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        logout::logout(self, request).await
    }

    /// Handles new user registration
    ///
    /// # Arguments
    /// * `request` - Registration request with user details
    ///
    /// # Returns
    /// A new session for the registered user
    async fn register(&self, request: Request<AuthRequest>) -> Result<Response<Session>, Status> {
        register::register(self, request).await
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

        let service = AuthService::new(db);

        assert!(matches!(service.db, Database::Mock(_)));
    }

    #[tokio::test]
    async fn test_service_builder() {
        let mock_db = MockDatabaseOperations::new();
        let db = Database::Mock(mock_db);

        let service = AuthService::build(db);

        assert!(matches!(service, AuthServiceServer { .. }));
    }
}
