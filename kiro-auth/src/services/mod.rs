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

use tonic::{async_trait, Request, Response, Status};

use kiro_api::{
    auth::v1::{
        auth_service_server::{self, AuthServiceServer},
        LoginRequest, RegisterRequest, Session,
    },
    google::protobuf::Empty,
};
use kiro_database::db_bridge::Database;

// mod login;
// mod logout;
// mod register;

#[derive(Clone)]
pub struct AuthService {
    pub db: Database,
}

impl AuthService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn build(db: Database) -> AuthServiceServer<Self> {
        AuthServiceServer::new(Self::new(db))
    }
}

#[async_trait]
impl auth_service_server::AuthService for AuthService {
    async fn login(&self, _request: Request<LoginRequest>) -> Result<Response<Session>, Status> {
        // login::login(self, request).await
        unimplemented!()
    }

    async fn logout(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        // logout::logout(self, request).await
        unimplemented!()
    }

    async fn register(
        &self, _request: Request<RegisterRequest>,
    ) -> Result<Response<Session>, Status> {
        // register::register(self, request).await
        unimplemented!()
    }
}
