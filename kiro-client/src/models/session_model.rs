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

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE, Engine};
use chrono::Utc;
use kiro_database::{
    db_bridge::{DatabaseOperations, HasId},
    get_env_or, DatabaseError, DbDateTime, DbId,
};
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[cfg(feature = "mailer")]
use kiro_mailer::{ContentType, Mailer, MailerTrait};

use crate::error::ClientError;

use super::UserModel;

static ENCRYPTION_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
});

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
///   pub session_key: String,
///   pub expires_at: Datetime,
///   pub user_id: Thing,
///   pub ip_address: Option<String>,
///   pub is_admin: bool,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: DbId,
    pub session_key: String,
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
///   pub session_key: String,
///   pub user_id: Thing,
///   pub is_admin: bool,
///   pub ip_address: Option<String>,
/// }
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct CreateSessionModel {
    pub session_key: String,
    pub user_id: DbId,
    pub is_admin: bool,
    pub ip_address: Option<String>,
}

impl HasId for SessionModel {
    type Id = DbId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl SessionModel {
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

    /// Encrypt user ID using AES-GCM
    fn encrypt_user_id(user_id: &DbId, key: &[u8; 32]) -> Result<String, ClientError> {
        let cipher = Aes256Gcm::new(key.into());

        // Generate random nonce
        let mut rng = thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt user ID
        let user_id_string = DbId::to_string(&user_id);
        let user_id_bytes = user_id_string.as_bytes();
        let ciphertext = cipher
            .encrypt(nonce, user_id_bytes.as_ref())
            .map_err(|_| ClientError::EncryptionError)?;

        // Combine nonce and ciphertext and encode
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(URL_SAFE.encode(combined))
    }

    /// Decrypt user ID using AES-GCM
    fn decrypt_user_id(encrypted: &str, key: &[u8; 32]) -> Result<DbId, ClientError> {
        let combined = URL_SAFE
            .decode(encrypted)
            .map_err(|_| ClientError::DecryptionError)?;

        if combined.len() < 12 {
            return Err(ClientError::DecryptionError);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(key.into());

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| ClientError::DecryptionError)?;

        let user_id = String::from_utf8(plaintext).map_err(|_| ClientError::DecryptionError)?;

        DbId::try_from(user_id).map_err(|_| ClientError::DecryptionError)
    }

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
    ) -> Result<SessionModel, ClientError> {
        let (session_key, _encrypted_user_id) =
            Self::generate_refresh_token(user_id.clone()).await?;

        db.create::<CreateSessionModel, SessionModel>(
            "sessions",
            CreateSessionModel {
                session_key: session_key.clone(),
                user_id,
                is_admin,
                ip_address,
            },
        )
        .await
        .map_err(ClientError::Database)
        .and_then(|res| res.first().cloned().ok_or(ClientError::NotCreated))
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
        db: &DB, encrypted_user_id: String,
    ) -> Result<Option<SessionModel>, ClientError> {
        let key = &*ENCRYPTION_KEY;
        let user_id = Self::decrypt_user_id(&encrypted_user_id, key)?;

        let res = db
            .read_by_field_thing::<SessionModel>("sessions", "user_id", user_id, None)
            .await?;

        match res.first() {
            Some(session) => {
                if Self::is_expired(&session.expires_at) {
                    Self::delete_session(db, session.id.clone()).await?;
                    Ok(None)
                } else {
                    Self::renew_session(db, session.id.clone()).await?;
                    Ok(Some(session.clone()))
                }
            }
            None => Ok(None),
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
    pub async fn get_session_by_user_id<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, ip_address: String,
    ) -> Result<SessionModel, ClientError> {
        let res = db
            .read_by_field_thing::<SessionModel>("sessions", "user_id", user_id.clone(), None)
            .await
            .map_err(ClientError::Database)?;

        if let Some(existing_session) = res.first() {
            #[cfg(feature = "mailer")]
            if existing_session.ip_address.as_deref() != Some(ip_address.as_str()) {
                // Get user
                let user = db
                    .select::<UserModel>(user_id.clone())
                    .await
                    .map_err(ClientError::Database)?
                    .ok_or(ClientError::NotFound)?;

                // Send new connection email
                let template = Mailer::load_template("new_connection_detected.html")
                    .await
                    .map_err(|e| DatabaseError::Internal(e.to_string()))?
                    .replace("${{CONNECTION_TYPE}}", "login")
                    .replace("${{CONNECTION_DATE}}", &chrono::Local::now().to_string())
                    .replace("${{CONNECTION_IP}}", &ip_address);

                let from = get_env_or("SMTP_USER", "contact@test.com");
                let to = user.email.clone();

                let message = Mailer::build_mail(
                    &from,
                    &to,
                    "New connection detected",
                    ContentType::TEXT_HTML,
                    template,
                )?;

                Mailer::new().send_mail(message).await.map(|_| ())?;
            }

            if Self::is_expired(&existing_session.expires_at) {
                Self::delete_session(db, existing_session.id.clone()).await?;
                return Err(ClientError::Expired);
            }

            // if ip_address is different, delete session and create new one
            if existing_session.ip_address.as_deref() != Some(ip_address.as_str()) {
                Self::delete_session(db, existing_session.id.clone()).await?;
                return Self::create_session(
                    db,
                    user_id,
                    existing_session.is_admin,
                    Some(ip_address),
                )
                .await;
            }

            Self::renew_session(db, existing_session.id.clone()).await?;

            Ok(existing_session.clone())
        } else {
            // Create new session with IP address
            Self::create_session(db, user_id, false, Some(ip_address)).await
        }
    }

    /// Generate refresh token with encrypted user ID
    pub async fn generate_refresh_token(user_id: DbId) -> Result<(String, String), ClientError> {
        // Use the static key
        let key = &*ENCRYPTION_KEY;

        // Rest of the implementation remains the same
        let session_key: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let encrypted_user_id = Self::encrypt_user_id(&user_id, key)?;

        Ok((session_key, encrypted_user_id))
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
        db: &DB, session_id: DbId,
    ) -> Result<(), ClientError> {
        db.delete(session_id)
            .await
            .map_err(ClientError::Database)
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
        db: &DB, session_id: DbId,
    ) -> Result<(), ClientError> {
        db.update_field(
            session_id,
            "expires_at",
            DbDateTime::from_timestamp(Utc::now().timestamp() + 2 * 24 * 60 * 60, 0).unwrap(),
        )
        .await
        .map_err(ClientError::Database)
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
    pub async fn create_password_hash(password: String) -> Result<String, ClientError> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| ClientError::PasswordHashingFailed)
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
    pub async fn verify_password(
        password: String, password_hash: String,
    ) -> Result<bool, ClientError> {
        let argon2 = Argon2::default();

        let hash = PasswordHash::new(&password_hash).map_err(|_| ClientError::PasswordIncorrect)?;

        match argon2.verify_password(password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
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
    ) -> Result<(), ClientError> {
        db.query::<SessionModel>("DELETE sessions;", None)
            .await
            .map_err(ClientError::Database)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use kiro_database::{db_bridge::MockDatabaseOperations, DbDateTime};
    #[cfg(feature = "mailer")]
    use kiro_mailer::{Category, Code, Detail, MockMailerTrait, Response, Severity};
    use mockall::predicate::*;

    use crate::models::user_model::{
        Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme, UserSettings,
    };

    /// Helper function to create a sample SessionModel for testing
    fn create_test_session() -> SessionModel {
        SessionModel {
            id: DbId::from(("sessions", "123")),
            session_key: "test_session_key".to_string(),
            expires_at: DbDateTime::from(Utc::now() + chrono::Duration::hours(48)),
            user_id: DbId::from(("users", "456")),
            ip_address: Some("127.0.0.1".to_string()),
            is_admin: false,
        }
    }

    /// Helper function to create a sample UserModel for testing
    #[cfg(feature = "mailer")]
    fn create_test_user() -> UserModel {
        UserModel {
            id: DbId::from(("users", "123")),
            customer_id: Some("cust_123".to_string()),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            avatar: Some("avatar.jpg".to_string()),
            settings: UserSettings {
                language: Some(Language::English),
                theme: Some(Theme::Dark),
                notifications: NotificationSettings {
                    email: true,
                    push: true,
                    sms: false,
                },
                privacy: PrivacySettings {
                    data_collection: true,
                    location: false,
                },
                security: SecuritySettings {
                    two_factor: true,
                    qr_code: "qr_code".to_string(),
                    magic_link: true,
                },
            },
            groups: vec![DbId::from(("groups", "123"))],
            created_at: DbDateTime::from(Utc::now()),
            updated_at: DbDateTime::from(Utc::now()),
            activated: true,
            is_admin: false,
        }
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();

        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .withf(|table: &str, _| table == "sessions")
            .times(1)
            .returning(move |_, _| Ok(vec![test_session.clone()]));

        let test_session = create_test_session();

        let result = SessionModel::create_session(
            &mock_db,
            test_session.user_id.clone(),
            false,
            Some("127.0.0.1".to_string()),
        )
        .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.user_id, test_session.user_id);
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
    }

    #[tokio::test]
    async fn test_get_session_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();
        let test_id = test_session.id.clone();
        let test_user_id = test_session.user_id.clone();

        // First, encrypt a user ID
        let (_, encrypted_user_id) = SessionModel::generate_refresh_token(test_user_id.clone())
            .await
            .unwrap();

        // Expect session lookup
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .withf(move |table: &str, field: &str, id: &DbId, _| {
                table == "sessions" && field == "user_id" && *id == test_user_id
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_session.clone()]));

        // Expect session renewal
        mock_db
            .expect_update_field::<DbDateTime>()
            .withf(move |id: &DbId, field: &str, value: &DbDateTime| {
                let expected_expiration = Utc::now().timestamp() + 2 * 24 * 60 * 60;
                let actual_expiration = value.timestamp();

                *id == test_id
                    && field == "expires_at"
                    && (actual_expiration - expected_expiration).abs() < 2
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let test_session = create_test_session();
        let test_user_id = test_session.user_id.clone();

        let result = SessionModel::get_session(&mock_db, encrypted_user_id).await;

        assert!(result.is_ok());
        let session_option = result.unwrap();
        assert!(session_option.is_some());
        let session = session_option.unwrap();
        assert_eq!(session.user_id, test_user_id);
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user_id = DbId::from(("users", "456"));

        // First, encrypt a user ID
        let (_, encrypted_user_id) = SessionModel::generate_refresh_token(test_user_id.clone())
            .await
            .unwrap();

        // Expect session lookup with empty result
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .withf(move |table: &str, field: &str, id: &DbId, _| {
                table == "sessions" && field == "user_id" && *id == test_user_id
            })
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let result = SessionModel::get_session(&mock_db, encrypted_user_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_session_expired() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut test_session = create_test_session();
        test_session.expires_at = DbDateTime::from(Utc::now() - chrono::Duration::hours(1));
        let test_id = test_session.id.clone();
        let test_user_id = test_session.user_id.clone();

        // First, encrypt a user ID
        let (_, encrypted_user_id) = SessionModel::generate_refresh_token(test_user_id.clone())
            .await
            .unwrap();

        // Expect session lookup
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .withf(move |table: &str, field: &str, id: &DbId, _| {
                table == "sessions" && field == "user_id" && *id == test_user_id
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_session.clone()]));

        // Expect delete call for expired session
        mock_db
            .expect_delete()
            .withf(move |id| *id == test_id)
            .times(1)
            .returning(|_| Ok(Some(())));

        let result = SessionModel::get_session(&mock_db, encrypted_user_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_session_invalid_encrypted_id() {
        let mock_db = MockDatabaseOperations::new();

        let result = SessionModel::get_session(&mock_db, "invalid_encrypted_id".to_string()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClientError::DecryptionError));
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let user_id = DbId::from(("users", "test_user"));
        let key = &*ENCRYPTION_KEY;

        // Test encryption
        let encrypted = SessionModel::encrypt_user_id(&user_id, key).unwrap();
        assert!(!encrypted.is_empty());

        // Test decryption
        let decrypted = SessionModel::decrypt_user_id(&encrypted, key).unwrap();
        assert_eq!(decrypted, user_id);
    }

    #[tokio::test]
    async fn test_decrypt_invalid_data() {
        let key = &*ENCRYPTION_KEY;
        let result = SessionModel::decrypt_user_id("invalid_data", key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClientError::DecryptionError));
    }

    #[tokio::test]
    async fn test_password_hash_verification() {
        let password = "test_password".to_string();

        // Create hash
        let hash = SessionModel::create_password_hash(password.clone())
            .await
            .unwrap();
        assert!(!hash.is_empty());

        // Verify correct password
        let result = SessionModel::verify_password(password.clone(), hash.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify incorrect password
        let result = SessionModel::verify_password("wrong_password".to_string(), hash).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_destroy_all_sessions() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_query::<SessionModel>()
            .withf(|query: &str, _| query == "DELETE sessions;")
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let result = SessionModel::destroy_all_sessions(&mock_db).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_session_by_user_id_existing_same_ip() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();
        let test_id = test_session.id.clone();
        let test_user_id = test_session.user_id.clone();

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .withf(move |table, field, id, _| {
                table == "sessions" && field == "user_id" && *id == test_user_id
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_session.clone()]));

        // Expect renew_session call
        mock_db
            .expect_update_field::<DbDateTime>()
            .withf(move |id, field, _| *id == test_id && field == "expires_at")
            .times(1)
            .returning(|_, _, _| Ok(()));

        let test_session = create_test_session();
        let test_user_id = test_session.user_id.clone();

        let result = SessionModel::get_session_by_user_id(
            &mock_db,
            test_user_id.clone(),
            "127.0.0.1".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.user_id, test_user_id);
    }

    #[tokio::test]
    #[cfg_attr(feature = "mailer", ignore)]
    async fn test_get_session_by_user_id_different_ip() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut test_session = create_test_session();
        test_session.ip_address = Some("192.168.1.1".to_string());
        let test_id = test_session.id.clone();
        let test_user_id = test_session.user_id.clone();
        let is_admin = test_session.is_admin;

        // First query to find existing session
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .withf(move |table, field, id, _| {
                table == "sessions" && field == "user_id" && *id == test_user_id
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_session.clone()]));

        // TODO: Need to fix this test to work with mailer
        #[cfg(feature = "mailer")]
        {
            let test_session = create_test_session();
            let test_user_id = test_session.user_id.clone();

            // Set up expectation for user lookup
            let test_user = create_test_user();
            mock_db
                .expect_select::<UserModel>()
                .with(eq(test_user_id.clone()))
                .times(1)
                .returning(move |_| Ok(Some(test_user.clone())));

            std::env::set_var("SMTP_HOST", "localhost");
            std::env::set_var("SMTP_USER", "test@example.com");
            std::env::set_var("SMTP_PASS", "password");

            // Setup mock mailer
            let mut mock_mailer = MockMailerTrait::default();
            mock_mailer
                .expect_send_mail()
                .withf(|_| true)
                .times(1)
                .returning(|_| {
                    Ok(Response::new(
                        Code::new(
                            Severity::PositiveCompletion,
                            Category::MailSystem,
                            Detail::Zero,
                        ),
                        Vec::new(),
                    ))
                });
        }

        // Expect delete_session call
        mock_db
            .expect_delete()
            .withf(move |id| *id == test_id)
            .times(1)
            .returning(|_| Ok(Some(())));

        let mut test_session = create_test_session();
        test_session.ip_address = Some("192.168.1.1".to_string());
        let test_user_id = test_session.user_id.clone();

        // Expect create_session call
        let new_session = create_test_session();
        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .withf(move |table, create_model| {
                table == "sessions"
                    && create_model.user_id == test_user_id
                    && create_model.is_admin == is_admin
                    && create_model.ip_address == Some("127.0.0.1".to_string())
            })
            .times(1)
            .returning(move |_, _| Ok(vec![new_session.clone()]));

        let mut test_session = create_test_session();
        test_session.ip_address = Some("192.168.1.1".to_string());
        let test_user_id = test_session.user_id.clone();

        let result =
            SessionModel::get_session_by_user_id(&mock_db, test_user_id, "127.0.0.1".to_string())
                .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_session_by_user_id_expired() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut test_session = create_test_session();
        test_session.expires_at = DbDateTime::from(Utc::now() - chrono::Duration::hours(1));
        let test_id = test_session.id.clone();
        let test_user_id = test_session.user_id.clone();

        // First query to find existing session
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_session.clone()]));

        // Expect delete_session call for expired session
        mock_db
            .expect_delete()
            .withf(move |id| *id == test_id)
            .times(1)
            .returning(|_| Ok(Some(())));

        let result =
            SessionModel::get_session_by_user_id(&mock_db, test_user_id, "127.0.0.1".to_string())
                .await;

        assert!(matches!(result, Err(ClientError::Expired)));
    }

    #[tokio::test]
    async fn test_get_session_by_user_id_new() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user_id = DbId::from(("users", "456"));

        // First query returns no existing session
        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        // Expect create_session call
        let new_session = create_test_session();
        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .withf(move |table, create_model| {
                table == "sessions"
                    && create_model.user_id == test_user_id
                    && !create_model.is_admin
                    && create_model.ip_address == Some("127.0.0.1".to_string())
            })
            .times(1)
            .returning(move |_, _| Ok(vec![new_session.clone()]));

        let test_user_id = DbId::from(("users", "456"));

        let result =
            SessionModel::get_session_by_user_id(&mock_db, test_user_id, "127.0.0.1".to_string())
                .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_renew_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();
        let test_id = test_session.id.clone();

        mock_db
            .expect_update_field::<DbDateTime>()
            .withf(move |id: &DbId, field: &str, value: &DbDateTime| {
                let expected_expiration = Utc::now().timestamp() + 2 * 24 * 60 * 60;
                let actual_expiration = value.timestamp();

                *id == test_id  // Use the cloned ID
                && field == "expires_at"
                && (actual_expiration - expected_expiration).abs() < 2
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        let result = SessionModel::renew_session(&mock_db, test_session.id).await;
        assert!(result.is_ok());
    }
}
