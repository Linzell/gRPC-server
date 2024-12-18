// models/session_model.rs
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

use chrono::Utc;
use kiro_database::{
    db_bridge::{DatabaseOperations, HasId},
    DbDateTime, DbId,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "mailer")]
use kiro_mailer::{Mailer, MailerTrait};

use crate::error::SessionError;

/// # Session Model
///
/// The session model is a model that represents a session.
///
/// ## Model
///
/// ```rust
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct SessionModel {
///   pub id: Thing,
///   pub data: String,
///   pub expires_at: Datetime,
///   pub user_id: Thing,
///   pub ip_address: Option<String>,
///   pub is_admin: bool,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: DbId,
    pub data: String,
    pub expires_at: DbDateTime,
    pub user_id: DbId,
    pub ip_address: Option<String>,
    pub is_admin: bool,
}

/// # Create Session Model
///
/// The create session model is a model that represents a create session.
///
/// ## Model
///
/// ```rust
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct CreateSessionModel {
///   pub data: String,
///   pub user_id: Thing,
///   pub is_admin: bool,
///   pub ip_address: Option<String>,
/// }
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct CreateSessionModel {
    pub data: String,
    pub user_id: DbId,
    pub is_admin: bool,
    pub ip_address: Option<String>,
}

/// # Session store struct
///
/// The session store struct is a struct that represents a session store.
pub struct SessionStore {}

impl HasId for SessionModel {
    type Id = DbId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[allow(unused)]
impl SessionStore {
    /// # Create session
    ///
    /// The `create_session` method creates a session.
    ///
    /// ```rust
    /// let session = SessionStore::create_session(db.clone(), user_id, is_admin, ip_address).await?;
    ///
    /// println!("🗝️ Session: {:?}", session);
    /// ```
    pub async fn create_session<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, is_admin: bool, ip_address: Option<String>,
    ) -> Result<SessionModel, SessionError> {
        let session_key = generate_session_key(db).await?;

        db.create::<CreateSessionModel, SessionModel>(
            "sessions",
            CreateSessionModel {
                data: session_key.clone(),
                user_id,
                is_admin,
                ip_address,
            },
        )
        .await
        .map_err(SessionError::Database)
        .and_then(|res| res.first().cloned().ok_or(SessionError::NotCreated))
    }

    /// # Get session
    ///
    /// The `get_session` method gets a session.
    ///
    /// ```rust
    /// let session = SessionStore::get_session(db.clone(), session_key).await?;
    ///
    /// println!("🗝️ Session: {:?}", session);
    /// ```
    pub async fn get_session<DB: DatabaseOperations + Send + Sync>(
        db: &DB, session_key: String,
    ) -> Result<Option<SessionModel>, SessionError> {
        let res = db
            .read_by_field::<SessionModel>("sessions", "data", &session_key, None)
            .await?;

        match res.first() {
            Some(session) => {
                if Self::is_expired(&session.expires_at) {
                    Self::delete_session(db, session_key).await?;
                    Ok(None)
                } else {
                    Self::renew_session(db, session_key).await.map(Some)
                }
            }
            None => Ok(None),
        }
    }

    /// # Get session by user
    ///
    /// The `get_session_by_user` method gets a session by user.
    ///
    /// ```rust
    /// let session = SessionStore::get_session_by_user(db.clone(), user, is_admin, ip_address).await?;
    ///
    /// println!("🗝️ Session: {:?}", session);
    /// ```
    pub async fn get_session_by_user<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, user_name: String, email: String, is_admin: bool,
        ip_address: String,
    ) -> Result<SessionModel, SessionError> {
        let res: Vec<SessionModel> = db
            .read_by_field_thing::<SessionModel>("sessions", "user_id", user_id.clone(), None)
            .await?;

        if let Some(existing_session) = res.first() {
            #[cfg(feature = "mailer")]
            if existing_session.ip_address.as_deref() != Some(ip_address.as_str()) {
                // Send new connection email
                let template = Mailer::load_template("new_connection_detected.html", None)
                    .await
                    .map_err(SessionError::IO)?
                    .replace("${{USER_NAME}}", &user_name)
                    .replace("${{CONNECTION_TYPE}}", "login")
                    .replace("${{CONNECTION_DATE}}", &chrono::Local::now().to_string())
                    .replace("${{CONNECTION_IP}}", &ip_address);

                let message =
                    Mailer::build_mail(&email, "New connection", ContentType::TEXT_HTML, template)?;

                Mailer::new().send_mail(message).await?;
            }

            if Self::is_expired(&existing_session.expires_at) {
                Self::delete_session(db, existing_session.clone().data).await?;
                return Err(SessionError::Expired);
            }

            // if ip_address is different, delete session and create new one
            if existing_session.ip_address.as_deref() != Some(ip_address.as_str()) {
                Self::delete_session(db, existing_session.clone().data).await?;
                return Self::create_session(db, user_id.clone(), is_admin, Some(ip_address)).await;
            }

            Self::renew_session(db, existing_session.clone().data).await
        } else {
            // Create new session with IP address
            let session =
                Self::create_session(db, user_id.clone(), is_admin, Some(ip_address)).await?;
            Ok(session)
        }
    }

    /// # Get token by user id
    ///
    /// The `get_token_by_user_id` method gets a token by user id.
    ///
    /// ```rust
    /// let token = SessionStore::get_token_by_user_id(db.clone(), user_id).await?;
    ///
    /// println!("🗝️ Token: {:?}", token);
    /// ```
    pub async fn get_token_by_user_id<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId,
    ) -> Result<Option<String>, SessionError> {
        db.read_by_field_thing::<SessionModel>("sessions", "user_id", user_id, None)
            .await
            .map_err(SessionError::Database)
            .map(|res| res.first().map(|session| session.data.clone()))
    }

    /// # Delete session
    ///
    /// The `delete_session` method deletes a session.
    ///
    /// ```rust
    /// SessionStore::delete_session(db.clone(), session_key).await?;
    ///
    /// println!("🗝️ Session deleted");
    /// ```
    pub async fn delete_session<DB: DatabaseOperations + Send + Sync>(
        db: &DB, session_key: String,
    ) -> Result<(), SessionError> {
        let bindings = serde_json::json!({
            "session_key": session_key
        });

        db.query::<SessionModel>(
            "DELETE FROM sessions WHERE data = $session_key;",
            Some(bindings),
        )
        .await
        .map_err(SessionError::Database)
        .map(|_| ())
    }

    /// # Renew session
    ///
    /// The `renew_session` method renews a session.
    ///
    /// ```rust
    /// let session = SessionStore::renew_session(db.clone(), session_key).await?;
    ///
    /// println!("🗝️ Session renewed: {:?}", session);
    /// ```
    pub async fn renew_session<DB: DatabaseOperations + Send + Sync>(
        db: &DB, session_key: String,
    ) -> Result<SessionModel, SessionError> {
        db.renew_session::<SessionModel>(session_key)
            .await
            .map_err(SessionError::Database)
    }

    /// # Create password hash
    ///
    /// The `create_password_hash` method creates a password hash.
    ///
    /// This method use Argon2 to hash the password.
    ///
    /// ```rust
    /// let password_hash = SessionStore::create_password_hash(db.clone(), password).await?;
    ///
    /// prin
    pub async fn create_password_hash<DB: DatabaseOperations + Send + Sync>(
        db: &DB, password: String,
    ) -> Result<String, SessionError> {
        let bindings = serde_json::json!({
            "password": password
        });

        db.query::<String>(
            "RETURN crypto::argon2::generate($password);",
            Some(bindings),
        )
        .await
        .map_err(SessionError::Database)
        .and_then(|res| {
            res.first()
                .cloned()
                .ok_or(SessionError::PasswordHashingFailed)
        })
    }

    /// # Verify password
    ///
    /// The `verify_password` method verifies a password.
    ///
    /// This method use Argon2 to verify the password.
    ///
    /// ```rust
    /// let is_valid = SessionStore::verify_password(db.clone(), password, password_hash).await?;
    ///
    /// println!("🔒 Password is valid: {:?}", is_valid);
    /// ```
    pub async fn verify_password<DB: DatabaseOperations + Send + Sync>(
        db: &DB, password: String, password_hash: String,
    ) -> Result<bool, SessionError> {
        let bindings = serde_json::json!({
            "password_hash": password_hash,
            "password": password
        });

        db.query::<bool>(
            "RETURN crypto::argon2::compare($password_hash, $password);",
            Some(bindings),
        )
        .await
        .map_err(SessionError::Database)
        .and_then(|res| res.first().copied().ok_or(SessionError::PasswordIncorrect))
    }

    /// # Destroy all sessions
    ///
    /// The `destroy_all_sessions` method destroys all sessions
    ///
    /// ```rust
    /// SessionStore::destroy_all_sessions(db.clone()).await?;
    ///
    /// println!("🗝️ All sessions destroyed");
    /// ```
    pub async fn destroy_all_sessions<DB: DatabaseOperations + Send + Sync>(
        db: &DB,
    ) -> Result<(), SessionError> {
        db.query::<SessionModel>("DELETE sessions;", None)
            .await
            .map_err(SessionError::Database)
            .map(|_| ())
    }

    /// # Check if session is expired
    ///
    /// The `is_expired` method checks if a session is expired.
    ///
    /// ```rust
    /// let is_expired = SessionStore::is_expired(&expires_at);
    ///
    /// println!("🗝️ Session expired: {:?}", is_expired);
    /// ```
    fn is_expired(expires_at: &DbDateTime) -> bool {
        let expiration = expires_at.timestamp();
        Utc::now().timestamp() > expiration
    }
}

/// # Generate session key
///
/// The `generate_session_key` method generates a session key.
///
/// This method use sha256 to generate the session key.
///
/// ```rust
/// let session_key = SessionStore::generate_session_key(db.clone(), user_id).await?;
///
/// println!("🗝️ Session key: {:?}", session_key);
/// ```
async fn generate_session_key<DB: DatabaseOperations + Send + Sync>(
    db: &DB,
) -> Result<String, SessionError> {
    db.query::<String>("RETURN crypto::sha256(rand::string(50));", None)
        .await
        .map_err(SessionError::Database)
        .and_then(|res| {
            res.first()
                .cloned()
                .ok_or(SessionError::KeyGenerationFailed)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use kiro_database::db_bridge::{Database, MockDatabaseOperations};
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_create_session() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();

        // 2. Set up expectations
        mock_db
            .expect_query::<String>()
            .with(eq("RETURN crypto::sha256(rand::string(50));"), eq(None))
            .times(1)
            .returning(|_, _| Ok(vec!["session_key".to_string()]));

        let expected_session = SessionModel {
            id: DbId::from(("sessions", "1")),
            data: "session_key".to_string(),
            expires_at: DbDateTime::from(Utc::now() + chrono::Duration::days(2)),
            user_id: DbId::from(("users", "1")),
            ip_address: Some("127.0.0.1".to_string()),
            is_admin: false,
        };

        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .returning(move |_, _| Ok(vec![expected_session.clone()]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::create_session(
            &db,
            DbId::from(("users", "1")),
            false,
            Some("127.0.0.1".to_string()),
        )
        .await;

        // 5. Verify results
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.data, "session_key");
        assert_eq!(session.user_id, DbId::from(("users", "1")));
        assert_eq!(session.is_admin, false);
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
    }

    #[tokio::test]
    async fn test_get_session() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();
        let session_key = "session_key".to_string();
        let expires_at = DbDateTime::from(Utc::now() + chrono::Duration::days(2));

        // 2. Set up expectations
        let expected_session = SessionModel {
            id: DbId::from(("sessions", "1")),
            data: session_key.clone(),
            expires_at: expires_at.clone(),
            user_id: DbId::from(("users", "1")),
            ip_address: Some("127.0.0.1".to_string()),
            is_admin: false,
        };

        mock_db
            .expect_read_by_field::<SessionModel>()
            .with(
                eq("sessions"),
                eq("data"),
                eq(session_key.clone()),
                eq(None),
            )
            .times(1)
            .returning({
                let expected_session = expected_session.clone();
                move |_, _, _, _| Ok(vec![expected_session.clone()])
            });

        mock_db
            .expect_renew_session::<SessionModel>()
            .with(eq(session_key.clone()))
            .times(1)
            .returning({
                let expected_session = expected_session.clone();
                move |_| Ok(expected_session.clone())
            });

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::get_session(&db, session_key).await;

        // 5. Verify results
        assert!(result.is_ok());
        let session = result.unwrap().unwrap();
        assert_eq!(session.id, DbId::from(("sessions", "1")));
    }

    #[tokio::test]
    async fn test_get_session_by_user() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();
        let is_admin = false;
        let user_id = DbId::from(("users", "1"));
        let user_name = "user_name".to_string();
        let email = "test@test.com".to_string();
        let ip_address = "127.0.0.1".to_string();
        let expires_at = DbDateTime::from(Utc::now() + chrono::Duration::days(2));

        // 2. Set up expectations
        let existing_session = SessionModel {
            id: DbId::from(("sessions", "1")),
            data: "session_key".to_string(),
            expires_at: expires_at.clone(),
            user_id: user_id.clone(),
            ip_address: Some(ip_address.clone()),
            is_admin,
        };

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(eq("sessions"), eq("user_id"), eq(user_id.clone()), eq(None))
            .times(1)
            .returning({
                let existing_session = existing_session.clone();
                move |_, _, _, _| Ok(vec![existing_session.clone()])
            });

        mock_db
            .expect_renew_session::<SessionModel>()
            .with(eq("session_key".to_string()))
            .times(1)
            .returning({
                let existing_session = existing_session.clone();
                move |_| Ok(existing_session.clone())
            });

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result =
            SessionStore::get_session_by_user(&db, user_id, user_name, email, is_admin, ip_address)
                .await;

        // 5. Verify results
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.id, DbId::from(("sessions", "1")));
    }

    #[tokio::test]
    async fn test_delete_session() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();
        let session_key = "session_key".to_string();

        // 2. Set up expectations
        mock_db
            .expect_query::<SessionModel>()
            .with(
                eq("DELETE FROM sessions WHERE data = $session_key;"),
                always(),
            )
            .times(1)
            .returning(|_, _| Ok(vec![]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::delete_session(&db, session_key).await;

        // 5. Verify results
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_password_hash() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();
        let password = "password123".to_string();

        // 2. Set up expectations
        mock_db
            .expect_query::<String>()
            .with(eq("RETURN crypto::argon2::generate($password);"), always())
            .times(1)
            .returning(|_, _| Ok(vec!["hashed_password".to_string()]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::create_password_hash(&db, password).await;

        // 5. Verify results
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash, "hashed_password");
    }

    #[tokio::test]
    async fn test_verify_password() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();
        let password = "password123".to_string();
        let password_hash = "hashed_password".to_string();

        // 2. Set up expectations
        mock_db
            .expect_query::<bool>()
            .with(
                eq("RETURN crypto::argon2::compare($password_hash, $password);"),
                always(),
            )
            .times(1)
            .returning(|_, _| Ok(vec![true]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::verify_password(&db, password, password_hash).await;

        // 5. Verify results
        assert!(result.is_ok());
        let is_valid = result.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_destroy_all_sessions() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();

        // 2. Set up expectations
        mock_db
            .expect_query::<SessionModel>()
            .with(eq("DELETE sessions;"), eq(None))
            .times(1)
            .returning(|_, _| Ok(vec![]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = SessionStore::destroy_all_sessions(&db).await;

        // 5. Verify results
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_session_key() {
        // 1. Create mock database
        let mut mock_db = MockDatabaseOperations::new();

        // 2. Set up expectations
        let expected_key = "generated_session_key".to_string();
        mock_db
            .expect_query::<String>()
            .with(eq("RETURN crypto::sha256(rand::string(50));"), eq(None))
            .times(1)
            .returning(move |_, _| Ok(vec![expected_key.clone()]));

        // 3. Create service with mock
        let db = Database::Mock(mock_db);

        // 4. Execute method
        let result = generate_session_key(&db).await;

        // 5. Verify results
        assert!(result.is_ok());
        let session_key = result.unwrap();
        assert_eq!(session_key, "generated_session_key");
    }
}
