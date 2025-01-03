// models/user/user_model.rs
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

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::Utc;
use kiro_api::client::v1::{Notifications, Privacy, Security, Settings, User};
use kiro_database::{
    db_bridge::{DatabaseOperations, HasId},
    DbDateTime, DbId,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::ClientError;

/// Represents available language options for user interface
///
/// Enum variants represent different supported languages with their corresponding
/// integer values for database storage and API communication.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    English = 0,
    Spanish = 1,
    French = 2,
    German = 3,
    Italian = 4,
    Japanese = 5,
    Korean = 6,
    Chinese = 7,
    Russian = 8,
    Arabic = 9,
}

/// Represents the user's theme preference for the application interface
///
/// Provides options for light mode, dark mode, or system-based theme selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    Light = 0,
    Dark = 1,
    System = 2,
}

/// # Represents user settings related to notifications
///
/// Controls different notification channels including email, push notifications,
/// and SMS messages
///
/// # Model
///
/// ```rust,no_run
/// use kiro_client::NotificationSettings;
///
/// let settings = NotificationSettings {
///     email: true,
///     push: true,
///     sms: false,
/// };
///
/// println!("{:?}", settings);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable/disable email notifications
    pub email: bool,
    /// Enable/disable push notifications
    pub push: bool,
    /// Enable/disable SMS notifications
    pub sms: bool,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email: true,
            push: true,
            sms: false,
        }
    }
}

impl From<&NotificationSettings> for Notifications {
    fn from(row: &NotificationSettings) -> Self {
        Self {
            email: row.email,
            push: row.push,
            sms: row.sms,
        }
    }
}

/// # Represents user privacy settings
///
/// Controls data collection and location tracking preferences
///
/// # Model
///
/// ```rust,no_run
/// use kiro_client::PrivacySettings;
///
/// let settings = PrivacySettings {
///     data_collection: true,
///     location: false,
/// };
///
/// println!("{:?}", settings);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Enable/disable data collection
    pub data_collection: bool,
    /// Enable/disable location tracking
    pub location: bool,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            data_collection: true,
            location: false,
        }
    }
}

impl From<&PrivacySettings> for Privacy {
    fn from(row: &PrivacySettings) -> Self {
        Self {
            data_collection: row.data_collection,
            location: row.location,
        }
    }
}

/// # Represents user security settings
///
/// Controls authentication and security related preferences
///
/// # Model
///
/// ```rust,no_run
/// use kiro_client::SecuritySettings;
///
/// let settings = SecuritySettings {
///     two_factor: true,
///     qr_code: "qr_code".to_string(),
///     magic_link: true,
/// };
///
/// println!("{:?}", settings);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable/disable two-factor authentication
    pub two_factor: bool,
    /// QR code for two-factor authentication setup
    pub qr_code: String,
    /// Enable/disable magic link authentication
    pub magic_link: bool,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            two_factor: true,
            qr_code: "qr_code".to_string(),
            magic_link: true,
        }
    }
}

impl From<&SecuritySettings> for Security {
    fn from(row: &SecuritySettings) -> Self {
        Self {
            two_factor: row.two_factor,
            qr_code: row.qr_code.clone(),
            magic_link: row.magic_link,
        }
    }
}

/// User Settings Model
///
/// Comprehensive settings model that represents all user preferences including:
/// - Language preference
/// - Theme selection
/// - Notification settings
/// - Privacy controls
/// - Security configurations
///
/// # Example
///
/// ```rust,no_run
/// use kiro_client::{Language, Theme, UserSettings, NotificationSettings, PrivacySettings, SecuritySettings};
///
/// let settings = UserSettings {
///     language: Some(Language::English),
///     theme: Some(Theme::Dark),
///     notifications: NotificationSettings { email: true, push: true, sms: false },
///     privacy: PrivacySettings { data_collection: true, location: false },
///     security: SecuritySettings {
///         two_factor: true,
///         qr_code: "qr_code".to_string(),
///         magic_link: true
///     },
/// };
///
/// println!("{:?}", settings);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub language: Option<Language>,
    pub theme: Option<Theme>,
    pub notifications: NotificationSettings,
    pub privacy: PrivacySettings,
    pub security: SecuritySettings,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for UserSettings {
    fn default() -> Self {
        Self {
            language: Some(Language::English),
            theme: Some(Theme::Dark),
            notifications: NotificationSettings::default(),
            privacy: PrivacySettings::default(),
            security: SecuritySettings::default(),
        }
    }
}

impl From<&UserSettings> for Settings {
    fn from(row: &UserSettings) -> Self {
        Self {
            language: row.language.map(|l| l as i32),
            theme: row.theme.map(|l| l as i32),
            notifications: Some(Notifications::from(&row.notifications)),
            privacy: Some(Privacy::from(&row.privacy)),
            security: Some(Security::from(&row.security)),
        }
    }
}

/// User Model
///
/// Core user model containing all user data and settings
///
/// # Fields
///
/// - `id`: Unique identifier for the user
/// - `customer_id`: Optional external customer reference
/// - `email`: User's email address (unique)
/// - `password_hash`: Hashed user password
/// - `avatar`: Optional URL to user's profile picture
/// - `settings`: User preferences and settings
/// - `groups`: List of group memberships
/// - `created_at`: Account creation timestamp
/// - `updated_at`: Last update timestamp
/// - `activated`: Account activation status
/// - `is_admin`: Administrative privileges flag
///
/// # Example
///
/// ```rust,no_run
/// use kiro_client::{UserModel, Language, Theme, UserSettings, NotificationSettings, PrivacySettings, SecuritySettings};
/// use kiro_database::{DbId, DbDateTime};
/// use chrono::Utc;
///
/// // Create new user model
/// let user = UserModel {
///     id: DbId::default(),
///     customer_id: Some("cust_123".to_string()),
///     email: "user@example.com".to_string(),
///     password_hash: "hashed_password".to_string(),
///     avatar: Some("avatar.jpg".to_string()),
///     settings: UserSettings {
///         language: Some(Language::English),
///         theme: Some(Theme::Dark),
///         notifications: NotificationSettings { email: true, push: true, sms: false },
///         privacy: PrivacySettings { data_collection: true, location: false },
///         security: SecuritySettings {
///             two_factor: true,
///             qr_code: "qr_code".to_string(),
///             magic_link: true,
///         }
///     },
///     groups: vec![],
///     created_at: DbDateTime::from(Utc::now()),
///     updated_at: DbDateTime::from(Utc::now()),
///     activated: true,
///     is_admin: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub id: DbId,
    pub customer_id: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub avatar: Option<String>,
    pub settings: UserSettings,
    pub groups: Vec<DbId>,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
    pub activated: bool,
    pub is_admin: bool,
}

impl HasId for UserModel {
    type Id = DbId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

// WARNING: This is a default implementation for testing purposes only
impl Default for UserModel {
    fn default() -> Self {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password("Password123!".as_bytes(), &salt)
            .unwrap()
            .to_string();

        Self {
            id: DbId::default(),
            customer_id: Some("cust_123".to_string()),
            email: "test@example.com".to_string(),
            password_hash,
            avatar: Some("avatar.jpg".to_string()),
            settings: UserSettings::default(),
            groups: vec![DbId::default()],
            created_at: DbDateTime::from(Utc::now()),
            updated_at: DbDateTime::from(Utc::now()),
            activated: true,
            is_admin: false,
        }
    }
}

impl From<&UserModel> for User {
    fn from(row: &UserModel) -> Self {
        Self {
            email: row.email.clone(),
            avatar: row.avatar.clone(),
            settings: Some(Settings::from(&row.settings)),
            is_admin: row.is_admin,
        }
    }
}

/// Create User Model
///
/// Data structure for creating new user accounts with minimal required information
///
/// # Fields
///
/// - `email`: User's email address
/// - `password_hash`: Hashed password for authentication
///
/// # Example
///
/// ```rust,no_run
/// use kiro_client::CreateUserModel;
///
/// // Create new user model
/// let new_user = kiro_client::CreateUserModel {
///     email: "new@example.com".to_string(),
///     password_hash: "secure_hash".to_string(),
/// };
///
/// println!("{:?}", new_user);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserModel {
    pub email: String,
    pub password_hash: String,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for CreateUserModel {
    fn default() -> Self {
        Self {
            email: "test@example.com".to_string(),
            password_hash: "secure_hash".to_string(),
        }
    }
}

impl UserModel {
    /// Get user by email
    ///
    /// Retrieves a user record from the database using their email address
    ///
    /// # Arguments
    /// * `db` - Database connection implementing DatabaseOperations trait
    /// * `email` - Email address to search for
    ///
    /// # Returns
    /// * `Ok(UserModel)` - User record if found
    /// * `Err(ClientError)` - Database error or user not found
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kiro_client::UserModel;
    /// use kiro_database::{DbId, db_bridge::{Database, MockDatabaseOperations}};
    ///
    /// // Mock database
    /// let db = Database::Mock(MockDatabaseOperations::new());
    ///
    /// // Mail
    /// let email = "user@example.com".to_string();
    ///
    /// // Async block to allow `await`
    /// tokio::runtime::Runtime::new().unwrap().block_on(async {
    ///    let user = UserModel::get_user_by_email(&db, email).await;
    ///
    ///    println!("{:?}", user);
    /// });
    /// ```
    pub async fn get_user_by_email<DB: DatabaseOperations + Send + Sync>(
        db: &DB, email: String,
    ) -> Result<Self, ClientError> {
        db.read_by_field::<Self>("users", "email", &email, None)
            .await
            .map_err(ClientError::Database)
            .and_then(|res| res.into_iter().next().ok_or(ClientError::DBOptionNone))
    }

    /// Check email availability
    ///
    /// Verifies if an email address is available for registration
    ///
    /// # Arguments
    /// * `db` - Database connection implementing DatabaseOperations trait
    /// * `email` - Email address to check
    ///
    /// # Returns
    /// * `Ok(true)` - Email is available
    /// * `Ok(false)` - Email is taken
    /// * `Err(ClientError)` - Database error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kiro_client::UserModel;
    /// use kiro_database::{DbId, db_bridge::{Database, MockDatabaseOperations}};
    ///
    /// // Mock database
    /// let db = Database::Mock(MockDatabaseOperations::new());
    ///
    /// // Mail
    /// let email = "user@example.com".to_string();
    ///
    /// // Async block to allow `await`
    /// tokio::runtime::Runtime::new().unwrap().block_on(async {
    ///    let available = UserModel::check_email(&db, email).await;
    ///
    ///    println!("{:?}", available);
    /// });
    /// ```
    pub async fn check_email<DB: DatabaseOperations + Send + Sync>(
        db: &DB, email: String,
    ) -> Result<bool, ClientError> {
        match Self::get_user_by_email(db, email.clone()).await {
            Ok(_) => Ok(false),
            Err(ClientError::DBOptionNone) => Ok(true),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_database::db_bridge::MockDatabaseOperations;
    use mockall::predicate::*;

    #[test]
    fn test_model_conversions() {
        let user_model = UserModel::default();

        // Test SecuritySettings conversion
        let security: Security = (&user_model.settings.security).into();
        assert_eq!(security.two_factor, true);
        assert_eq!(security.qr_code, "qr_code");
        assert_eq!(security.magic_link, true);

        // Test PrivacySettings conversion
        let privacy: Privacy = (&user_model.settings.privacy).into();
        assert_eq!(privacy.data_collection, true);
        assert_eq!(privacy.location, false);

        // Test NotificationSettings conversion
        let notifications: Notifications = (&user_model.settings.notifications).into();
        assert_eq!(notifications.email, true);
        assert_eq!(notifications.push, true);
        assert_eq!(notifications.sms, false);

        // Test UserSettings conversion
        let settings: Settings = (&user_model.settings).into();
        assert_eq!(settings.language, Some(Language::English as i32));
        assert_eq!(settings.theme, Some(Theme::Dark as i32));
        assert!(settings.notifications.is_some());
        assert!(settings.privacy.is_some());
        assert!(settings.security.is_some());

        // Test UserModel conversion to User
        let user: User = (&user_model).into();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.avatar, Some("avatar.jpg".to_string()));
        assert_eq!(user.is_admin, false);
        assert!(user.settings.is_some());
    }

    #[tokio::test]
    async fn test_get_user_by_email_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user = UserModel::default();
        let test_email = test_user.email.clone();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(move |table: &str, field: &str, value: &str, _| {
                table == "users" && field == "email" && value == test_email
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_user.clone()]));

        let test_user = UserModel::default();
        let test_email = test_user.email.clone();

        let result = UserModel::get_user_by_email(&mock_db, test_email).await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_by_email_not_found() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_email = "nonexistent@example.com".to_string();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(move |table: &str, field: &str, value: &str, _| {
                table == "users" && field == "email" && value == test_email
            })
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let test_email = "nonexistent@example.com".to_string();

        let result = UserModel::get_user_by_email(&mock_db, test_email).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClientError::DBOptionNone));
    }

    #[tokio::test]
    async fn test_check_email_exists() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user = UserModel::default();
        let test_email = test_user.email.clone();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(move |table: &str, field: &str, value: &str, _| {
                table == "users" && field == "email" && value == test_email
            })
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_user.clone()]));

        let test_user = UserModel::default();
        let test_email = test_user.email.clone();

        let result = UserModel::check_email(&mock_db, test_email).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_check_email_not_exists() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_email = "nonexistent@example.com".to_string();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(move |table: &str, field: &str, value: &str, _| {
                table == "users" && field == "email" && value == test_email
            })
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let test_email = "nonexistent@example.com".to_string();

        let result = UserModel::check_email(&mock_db, test_email).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
