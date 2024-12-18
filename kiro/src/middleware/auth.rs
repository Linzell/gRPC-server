// utils/auth.rs
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

use futures::future::BoxFuture;
use http::{Request, Response, StatusCode};
use kiro_auth::{SessionModel, SessionStore};
use kiro_database::db_bridge::DatabaseOperations;
use tonic::{metadata::MetadataMap, Status};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

use crate::{config::LoggingConfig, tonic_auth, utils::grpc_utils::get_token_from_md};

/// Authentication middleware for handling session validation and authorization
#[derive(Clone)]
pub struct AuthMiddleware<DB> {
    db: DB,
    config: LoggingConfig,
}

impl<DB> AuthMiddleware<DB>
where
    DB: DatabaseOperations + Clone + Send + Sync + 'static,
{
    /// Creates a new AuthMiddleware instance
    pub fn new(db: DB) -> Self {
        Self {
            db,
            config: LoggingConfig::default(),
        }
    }

    /// Converts HTTP headers to gRPC metadata
    fn http_headers_to_grpc_metadata(headers: &http::HeaderMap) -> MetadataMap {
        let mut metadata = MetadataMap::new();

        if headers.contains_key("access-control-request-method") {
            return metadata;
        }

        let auth = headers
            .get("authorization")
            .or_else(|| headers.get(http::header::AUTHORIZATION))
            .or_else(|| headers.get("Authorization"));

        if let Some(auth) = auth {
            if let Ok(auth_str) = auth.to_str() {
                if let Ok(value) = tonic::metadata::MetadataValue::try_from(auth_str) {
                    if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(b"authorization") {
                        metadata.insert(key, value);
                    }
                }
            }
        }

        metadata
    }

    /// Validates the session for a request
    async fn validate_session(&self, request: &Request<()>) -> Result<SessionModel, Status> {
        let path = request.uri().path();

        if request.method() == http::Method::OPTIONS {
            return Err(Status::ok("OPTIONS request"));
        }

        if self.config.public_endpoints.contains(&path.to_string()) {
            return Err(Status::ok("Public endpoint"));
        }

        let metadata = Self::http_headers_to_grpc_metadata(request.headers());
        let token = tonic_auth!(get_token_from_md(&metadata), "Token authentication error");

        match SessionStore::get_session(&self.db, token.clone()).await {
            Ok(Some(session)) => {
                if self.config.admin_endpoints.contains(&path.to_string()) && !session.is_admin {
                    return Err(Status::permission_denied("Admin privileges required"));
                }
                Ok(session)
            }
            Ok(None) => Err(Status::unauthenticated("Unauthorized: Invalid token")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

impl<DB, ResBody> AsyncAuthorizeRequest<ResBody> for AuthMiddleware<DB>
where
    DB: DatabaseOperations + Clone + Send + Sync + 'static,
    ResBody: Send + Default + 'static,
{
    type RequestBody = ResBody;
    type ResponseBody = ResBody;
    type Future = BoxFuture<'static, Result<Request<ResBody>, Response<ResBody>>>;

    fn authorize(&mut self, mut request: Request<ResBody>) -> Self::Future {
        let this = self.clone();

        Box::pin(async move {
            let path = request.uri().path();

            if request.method() == http::Method::OPTIONS {
                return Ok(request);
            }

            if this.config.excluded_paths.contains(&path.to_string()) {
                return Ok(request);
            }

            let mut builder = Request::builder()
                .uri(request.uri().clone())
                .method(request.method().clone());

            for (key, value) in request.headers() {
                builder = builder.header(key, value);
            }

            let validation_request = builder.body(()).map_err(|_| {
                Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(ResBody::default())
                    .unwrap()
            })?;

            match this.validate_session(&validation_request).await {
                Ok(session) => {
                    request.extensions_mut().insert(session);
                    Ok(request)
                }
                Err(status) if status.code() == tonic::Code::Ok => Ok(request),
                Err(status) => {
                    let status_code = match status.code() {
                        tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
                        tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
                        tonic::Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                        _ => StatusCode::UNAUTHORIZED,
                    };

                    let mut response = Response::builder()
                        .status(status_code)
                        .body(ResBody::default())
                        .map_err(|_| {
                            Response::builder()
                                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .body(ResBody::default())
                                .unwrap()
                        })?;

                    if status_code == StatusCode::UNAUTHORIZED {
                        response.headers_mut().insert(
                            http::header::WWW_AUTHENTICATE,
                            http::HeaderValue::from_static("Bearer error=\"invalid_token\""),
                        );
                    }

                    Err(response)
                }
            }
        })
    }
}

/// Creates an authentication middleware layer
pub fn auth_layer<DB>(db: DB) -> AsyncRequireAuthorizationLayer<AuthMiddleware<DB>>
where
    DB: DatabaseOperations + Clone + Send + Sync + 'static,
{
    AsyncRequireAuthorizationLayer::new(AuthMiddleware::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::HeaderValue;
    use kiro_database::db_bridge::MockDatabaseOperations;

    #[tokio::test]
    async fn test_http_headers_to_grpc_metadata() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );

        let metadata =
            AuthMiddleware::<MockDatabaseOperations>::http_headers_to_grpc_metadata(&headers);
        assert!(metadata.contains_key("authorization"));
    }

    #[test]
    fn test_auth_middleware_new() {
        let db = MockDatabaseOperations::new();
        let middleware = AuthMiddleware::new(db);
        assert!(middleware.config.public_endpoints.len() > 0);
    }
}
