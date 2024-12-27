// models/link_model.rs
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

use kiro_database::{db_bridge::HasId, DbDateTime, DbId};
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "client", feature = "group"))]
use chrono::{DateTime, Utc};
#[cfg(any(feature = "client", feature = "group"))]
use kiro_database::{
    db_bridge::{DatabaseOperations, OrderDirection, QueryOptions},
    get_env_or,
};

#[cfg(any(feature = "client", feature = "group"))]
use crate::error::MailerError;

/// # Link Type
///
/// The link type is an enum that represents a link type.
///
/// ## Enum
///
/// ```rust
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// pub enum LinkType {
///   Unimplemented,
///   EmailChange,
///   EmailReset,
///   PasswordChange,
///   PasswordReset,
///   VerifyAccount,
///   EmailGroupReset,
///   EmailGroupChange,
/// }
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    Unimplemented,
    #[cfg(feature = "client")]
    EmailChange,
    #[cfg(feature = "client")]
    PasswordChange,
    #[cfg(feature = "client")]
    PasswordReset,
    #[cfg(feature = "client")]
    VerifyAccount,
    #[cfg(feature = "group")]
    EmailGroupReset,
    #[cfg(feature = "group")]
    EmailGroupChange,
    #[cfg(any(feature = "client", feature = "group"))]
    EmailReset,
}

/// # Link Model
///
/// The link model is a model that represents a link.
///
/// ## Model
///
/// ```rust
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct LinkModel {
///   pub id: DbId,
///   pub user: DbId,
///   pub link_type: LinkType,
///   pub expiry: Datetime,
/// }
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LinkModel {
    pub id: DbId,
    pub user: DbId,
    pub link_type: LinkType,
    pub expiry: DbDateTime,
}

impl PartialEq for LinkModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.user == other.user
            && self.link_type == other.link_type
            && self.expiry.inner() == other.expiry.inner()
    }
}

/// # Create Link Model
///
/// The create link model is a model that represents a create link.
///
/// ## Model
///
/// ```rust
/// #[derive(Clone, Serialize, Deserialize)]
/// pub struct CreateLinkModel {
///   pub user: DbId,
///   pub link_type: LinkType,
///   pub expiry: Datetime,
/// }
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateLinkModel {
    pub user: DbId,
    pub link_type: LinkType,
    pub expiry: DbDateTime,
}

impl PartialEq for CreateLinkModel {
    fn eq(&self, other: &Self) -> bool {
        self.user == other.user
            && self.link_type == other.link_type
            && self.expiry.inner() == other.expiry.inner()
    }
}

impl HasId for LinkModel {
    type Id = DbId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl LinkModel {
    /// # Create from user
    ///
    /// The `create_from_user` method creates a link from a user.
    ///
    /// ```rust
    /// let link = Link::create_from_user(db.clone(), user_id, expiry_time, link_type).await?;
    ///
    /// println!("🔗 Link: {:?}", link);
    /// ```
    #[cfg(any(feature = "client", feature = "group"))]
    pub async fn create_from_user<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, expiry_time: DateTime<Utc>, link_type: LinkType,
    ) -> Result<Self, MailerError> {
        db.create::<CreateLinkModel, Self>(
            "links",
            CreateLinkModel {
                user: user_id,
                link_type,
                expiry: DbDateTime::from(expiry_time),
            },
        )
        .await
        .map_err(MailerError::Database)
        .and_then(|res| res.first().cloned().ok_or(MailerError::NotFound))
    }

    /// # Get links by user
    ///
    /// The `get_links_by_user` method gets a link by user.
    ///
    /// ```rust
    /// let link = Link::get_links_by_user(db.clone(), user_id).await?;
    ///
    /// println!("🔗 Link: {:?}", link);
    /// ```
    #[cfg(any(feature = "client", feature = "group"))]
    pub async fn get_links_by_user<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId,
    ) -> Result<Vec<Self>, MailerError> {
        db.read_by_field_thing::<LinkModel>(
            "links",
            "user",
            user_id,
            Some(QueryOptions {
                order_by: Some("expiry:".to_string()),
                order_direction: Some(OrderDirection::ASC),
                limit: Some(10),
            }),
        )
        .await
        .map_err(MailerError::Database)
        .map(|links| {
            links
                .into_iter()
                .filter(|link| link.expiry > Utc::now())
                .collect()
        })
    }

    /// # Get link by user and type
    ///
    /// The `get_link_by_user_and_type` method gets a link by user and type.
    ///
    /// ```rust
    /// let link = Link::get_link_by_user_and_type(db.clone(), user_id, link_type).await?;
    ///
    /// println!("🔗 Link: {:?}", link);
    /// ```
    #[cfg(any(feature = "client", feature = "group"))]
    pub async fn get_link_by_user_and_type<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, link_type: LinkType,
    ) -> Result<Self, MailerError> {
        db.read_by_field_thing::<LinkModel>(
            "links",
            "user",
            user_id,
            Some(QueryOptions {
                order_by: Some("expiry:".to_string()),
                order_direction: Some(OrderDirection::ASC),
                limit: Some(10),
            }),
        )
        .await
        .map_err(MailerError::Database)
        .and_then(|links| {
            links
                .into_iter()
                .find(|link| link.link_type == link_type && link.expiry > Utc::now())
                .ok_or(MailerError::NotFound)
        })
    }

    /// # Delete link by user and type
    ///
    /// The `delete_link_by_user_and_type` method deletes a link by user and type.
    ///
    /// ```rust
    /// Link::delete_link_by_user_and_type(db.clone(), user_id, link_type).await?;
    ///
    /// println!("🔗 Link deleted");
    /// ```
    #[cfg(any(feature = "client", feature = "group"))]
    pub async fn delete_link_by_user_and_type<DB: DatabaseOperations + Send + Sync>(
        db: &DB, user_id: DbId, link_type: LinkType,
    ) -> Result<(), MailerError> {
        let bindings = serde_json::json!({
            "user": user_id,
            "link_type": link_type
        });

        db.query::<LinkModel>(
            "DELETE links WHERE user = type::thing($user) AND link_type = $link_type;",
            Some(bindings),
        )
        .await
        .map_err(MailerError::Database)
        .and_then(|res| res.first().map(|_| ()).ok_or(MailerError::NotFound))
    }

    /// # Construct link
    ///
    /// The `construct_link` method constructs a link.
    ///
    /// ```rust
    /// let link_url = Link::construct_link(&self);
    ///
    /// println!("🔗 Link URL: {:?}", link_url);
    /// ```
    #[cfg(any(feature = "client", feature = "group"))]
    pub fn construct_link(&self) -> String {
        let typestr = match self.link_type {
            LinkType::Unimplemented => "unimplemented",
            #[cfg(feature = "client")]
            LinkType::EmailChange => "change-email",
            #[cfg(feature = "client")]
            LinkType::PasswordChange => "change-password",
            #[cfg(feature = "client")]
            LinkType::PasswordReset => "reset-password",
            #[cfg(feature = "client")]
            LinkType::VerifyAccount => "verify",
            #[cfg(feature = "group")]
            LinkType::EmailGroupChange => "change-group-email",
            #[cfg(feature = "group")]
            LinkType::EmailGroupReset => "reset-group-email",
            #[cfg(any(feature = "client", feature = "group"))]
            LinkType::EmailReset => "reset-email",
        };
        let url = get_env_or("FRONT_URL", "http://localhost:5173");

        format!("{}/{}/{}", url, typestr, self.id.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    #[cfg(any(feature = "client", feature = "group"))]
    use kiro_database::db_bridge::MockDatabaseOperations;
    #[cfg(any(feature = "client", feature = "group"))]
    use mockall::predicate::*;

    /// Helper function to create a sample LinkModel for testing
    #[cfg(any(feature = "client", feature = "group"))]
    fn create_test_link() -> LinkModel {
        LinkModel {
            id: DbId::from(("links", "123")),
            user: DbId::from(("users", "456")),
            #[cfg(any(feature = "client", feature = "group"))]
            link_type: LinkType::EmailReset,
            #[cfg(not(any(feature = "client", feature = "group")))]
            link_type: LinkType::Unimplemented,
            expiry: DbDateTime::from(Utc::now() + Duration::hours(24)),
        }
    }

    #[test]
    fn test_link_model_equality() {
        let expiry = DbDateTime::from(Utc::now() + Duration::hours(24));
        let link1 = LinkModel {
            id: DbId::from(("links", "123")),
            user: DbId::from(("users", "456")),
            #[cfg(any(feature = "client", feature = "group"))]
            link_type: LinkType::EmailReset,
            #[cfg(not(any(feature = "client", feature = "group")))]
            link_type: LinkType::Unimplemented,
            expiry: expiry.clone(),
        };
        let link2 = LinkModel {
            id: DbId::from(("links", "123")),
            user: DbId::from(("users", "456")),
            #[cfg(any(feature = "client", feature = "group"))]
            link_type: LinkType::EmailReset,
            #[cfg(not(any(feature = "client", feature = "group")))]
            link_type: LinkType::Unimplemented,
            expiry,
        };
        assert_eq!(link1, link2);
    }

    #[test]
    fn test_create_link_model_equality() {
        let expiry = DbDateTime::from(Utc::now() + Duration::hours(24));
        let create1 = CreateLinkModel {
            user: DbId::from(("users", "456")),
            #[cfg(any(feature = "client", feature = "group"))]
            link_type: LinkType::EmailReset,
            #[cfg(not(any(feature = "client", feature = "group")))]
            link_type: LinkType::Unimplemented,
            expiry: expiry.clone(),
        };
        let create2 = CreateLinkModel {
            user: DbId::from(("users", "456")),
            #[cfg(any(feature = "client", feature = "group"))]
            link_type: LinkType::EmailReset,
            #[cfg(not(any(feature = "client", feature = "group")))]
            link_type: LinkType::Unimplemented,
            expiry,
        };
        assert_eq!(create1, create2);
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[tokio::test]
    async fn test_create_from_user_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_link = create_test_link();
        let expiry_time = Utc::now() + Duration::hours(24);
        let test_user = test_link.user.clone();
        let test_link_type = test_link.link_type;
        let test_link_clone = test_link.clone();

        mock_db
            .expect_create::<CreateLinkModel, LinkModel>()
            .withf(move |table: &str, create_model: &CreateLinkModel| {
                table == "links"
                    && create_model.user == test_user
                    && create_model.link_type == test_link_type
            })
            .times(1)
            .returning(move |_, _| Ok(vec![test_link_clone.clone()]));

        let result = LinkModel::create_from_user(
            &mock_db,
            test_link.user.clone(),
            expiry_time,
            test_link.link_type.clone(),
        )
        .await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.user, test_link.user);
        assert_eq!(link.link_type, test_link.link_type);
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[tokio::test]
    async fn test_get_links_by_user_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_link = create_test_link();
        let test_user_id = test_link.user.clone();
        let test_link_clone = test_link.clone();
        let test_user_id_clone = test_user_id.clone();

        mock_db
            .expect_read_by_field_thing::<LinkModel>()
            .withf(
                move |table: &str, field: &str, user_id: &DbId, options: &Option<QueryOptions>| {
                    table == "links"
                        && field == "user"
                        && *user_id == test_user_id_clone
                        && options.is_some()
                },
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_link_clone.clone()]));

        let result = LinkModel::get_links_by_user(&mock_db, test_user_id).await;

        assert!(result.is_ok());
        let links = result.unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], test_link);
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[tokio::test]
    async fn test_get_link_by_user_and_type_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_link = create_test_link();
        let test_user_id = test_link.user.clone();
        let test_link_type = test_link.link_type.clone();
        let test_link_clone = test_link.clone();
        let test_user_id_clone = test_user_id.clone();
        let test_link_clone2 = test_link.clone();

        mock_db
            .expect_read_by_field_thing::<LinkModel>()
            .withf(
                move |table: &str, field: &str, user_id: &DbId, options: &Option<QueryOptions>| {
                    table == "links"
                        && field == "user"
                        && *user_id == test_user_id_clone
                        && options.is_some()
                },
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_link_clone.clone()]));

        let result =
            LinkModel::get_link_by_user_and_type(&mock_db, test_user_id, test_link_type).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link, test_link_clone2);
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[tokio::test]
    async fn test_delete_link_by_user_and_type_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_link = create_test_link();
        let test_user_id = test_link.user.clone();
        let test_link_type = test_link.link_type.clone();

        mock_db
            .expect_query::<LinkModel>()
            .withf(move |query: &str, bindings: &Option<serde_json::Value>| {
                query.contains("DELETE links WHERE") && bindings.is_some()
            })
            .times(1)
            .returning(move |_, _| Ok(vec![test_link.clone()]));

        let result =
            LinkModel::delete_link_by_user_and_type(&mock_db, test_user_id, test_link_type).await;

        assert!(result.is_ok());
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[test]
    fn test_construct_link() {
        std::env::set_var("FRONT_URL", "http://test.com");
        let test_link = create_test_link();

        let link_url = test_link.construct_link();

        assert!(link_url.starts_with("http://test.com/"));
        assert!(link_url.contains(&test_link.id.id.to_string()));
    }

    #[cfg(any(feature = "client", feature = "group"))]
    #[tokio::test]
    async fn test_get_links_by_user_expired_filtered() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut expired_link = create_test_link();
        expired_link.expiry = DbDateTime::from(Utc::now() - Duration::hours(1));
        let valid_link = create_test_link();
        let test_user_id = valid_link.user.clone();
        let valid_link_clone = valid_link.clone();

        mock_db
            .expect_read_by_field_thing::<LinkModel>()
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![expired_link.clone(), valid_link.clone()]));

        let result = LinkModel::get_links_by_user(&mock_db, test_user_id).await;

        assert!(result.is_ok());
        let links = result.unwrap();
        assert_eq!(links.len(), 1); // Only the valid link should be returned
        assert_eq!(links[0], valid_link_clone);
    }
}
