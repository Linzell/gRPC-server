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
use base64::{engine::general_purpose::URL_SAFE, Engine};
use chrono::Utc;
use kiro_database::{
    db_bridge::{DatabaseOperations, HasId},
    DbDateTime, DbId,
};
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[cfg(feature = "mailer")]
use kiro_mailer::{Mailer, MailerTrait};

use crate::error::SessionError;

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

impl SessionStore {
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
    fn encrypt_user_id(user_id: &DbId, key: &[u8; 32]) -> Result<String, SessionError> {
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
            .map_err(|_| SessionError::EncryptionError)?;

        // Combine nonce and ciphertext and encode
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(URL_SAFE.encode(combined))
    }

    /// Decrypt user ID using AES-GCM
    fn decrypt_user_id(encrypted: &str, key: &[u8; 32]) -> Result<DbId, SessionError> {
        let combined = URL_SAFE
            .decode(encrypted)
            .map_err(|_| SessionError::DecryptionError)?;

        if combined.len() < 12 {
            return Err(SessionError::DecryptionError);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(key.into());

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SessionError::DecryptionError)?;

        let user_id = String::from_utf8(plaintext).map_err(|_| SessionError::DecryptionError)?;

        DbId::try_from(user_id).map_err(|_| SessionError::DecryptionError)
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
    ) -> Result<SessionModel, SessionError> {
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
        db: &DB, encrypted_user_id: String,
    ) -> Result<Option<SessionModel>, SessionError> {
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
    ) -> Result<SessionModel, SessionError> {
        let res = db
            .read_by_field_thing::<SessionModel>("sessions", "user_id", user_id.clone(), None)
            .await
            .map_err(SessionError::Database)?;

        if let Some(existing_session) = res.first() {
            #[cfg(feature = "mailer")]
            if existing_session.ip_address.as_deref() != Some(ip_address.as_str()) {
                // Send new connection email
                let template = Mailer::load_template("new_connection_detected.html", None)
                    .await
                    .map_err(SessionError::IO)?
                    .replace("${{CONNECTION_TYPE}}", "login")
                    .replace("${{CONNECTION_DATE}}", &chrono::Local::now().to_string())
                    .replace("${{CONNECTION_IP}}", &ip_address);

                let message =
                    Mailer::build_mail("New connection", ContentType::TEXT_HTML, template)?;

                Mailer::new().send_mail(message).await?;
            }

            if Self::is_expired(&existing_session.expires_at) {
                Self::delete_session(db, existing_session.id.clone()).await?;
                return Err(SessionError::Expired);
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
    pub async fn generate_refresh_token(user_id: DbId) -> Result<(String, String), SessionError> {
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
    ) -> Result<(), SessionError> {
        db.delete(session_id)
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
        db: &DB, session_id: DbId,
    ) -> Result<(), SessionError> {
        db.update_field(
            session_id,
            "expires_at",
            DbDateTime::from_timestamp(Utc::now().timestamp() + 2 * 24 * 60 * 60, 0).unwrap(),
        )
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
}

#[cfg(test)]
mod tests {}
