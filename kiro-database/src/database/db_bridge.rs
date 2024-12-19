// database/db_bridge.rs
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

use std::fmt::Display;
use std::pin::Pin;

use async_trait::async_trait;
#[cfg(feature = "surrealdb")]
use futures::stream::StreamExt;
use futures::Stream;
use mockall::automock;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "surrealdb")]
use std::sync::Arc;
#[cfg(feature = "surrealdb")]
use tokio::sync::mpsc;
use tokio::sync::watch::Sender;
#[cfg(feature = "surrealdb")]
use tokio_stream::wrappers::ReceiverStream;

#[cfg(feature = "surrealdb")]
use surrealdb::{
    engine::any::Any,
    opt::PatchOp,
    sql::{Datetime, Thing},
    Surreal,
};

#[cfg(feature = "surrealdb")]
use chrono::Utc;

use crate::database::db_types::DbId;
use crate::error::DatabaseError;

pub type DatabaseStream<T> = Pin<Box<dyn Stream<Item = Result<T, DatabaseError>> + Send>>;

pub type QueryBindings = serde_json::Value;

#[derive(Clone)]
pub enum Database {
    #[cfg(feature = "surrealdb")]
    Surreal(Surreal<Any>),
    Mock(MockDatabaseOperations),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum DatabaseAction {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
pub struct DatabaseNotification<T> {
    pub action: DatabaseAction,
    pub data: T,
}

pub trait HasId: Clone {
    type Id: Clone + PartialEq + Display + Send + Sync;
    fn id(&self) -> &Self::Id;
}

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq)]
pub enum OrderDirection {
    ASC,
    DESC,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryOptions {
    pub order_by: Option<String>,
    pub order_direction: Option<OrderDirection>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrganizationRole {
    Owner,
    Moderator,
    User,
}

#[async_trait]
impl DatabaseOperations for Database {
    async fn create<T, U>(&self, table: &str, data: T) -> Result<Vec<U>, DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static,
        U: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => db
                .create(table)
                .content(data)
                .await
                .map_err(DatabaseError::from),
            Self::Mock(db) => db.create(table, data).await,
        }
    }

    async fn select<T>(&self, thing: DbId) -> Result<Option<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                let sql = String::from("SELECT * FROM type::thing($thing)");
                db.query(&sql)
                    .bind(("thing", surreal_thing))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.select(thing).await,
        }
    }

    async fn query<U>(
        &self, sql: &str, bindings: Option<QueryBindings>,
    ) -> Result<Vec<U>, DatabaseError>
    where
        U: for<'de> SerdeDeserialize<'de> + Send + Sync + Clone + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let query = db.query(sql);
                let result: Vec<U> = if let Some(bindings) = bindings {
                    query.bind(bindings).await?
                } else {
                    query.await?
                }
                .take(0)
                .map_err(DatabaseError::from)?;

                match result.len() {
                    0 => Ok(Vec::new()),
                    1 => Ok(vec![result[0].clone()]),
                    _ => Ok(result),
                }
            }
            Self::Mock(db) => db.query(sql, bindings).await,
        }
    }

    async fn update<T>(&self, thing: DbId, data: T) -> Result<Option<T>, DatabaseError>
    where
        T: SerdeSerialize + for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                db.update(surreal_thing)
                    .content(data)
                    .await
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.update(thing, data).await,
        }
    }

    async fn delete(&self, thing: DbId) -> Result<Option<()>, DatabaseError> {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                db.delete(surreal_thing).await.map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.delete(thing).await,
        }
    }

    async fn read_all<T>(
        &self, table: &str, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let mut sql = String::from("SELECT * FROM type::table($table)");

                if let Some(opts) = options {
                    if let Some(order_by) = opts.order_by {
                        sql.push_str(&format!(" ORDER BY {}", order_by));
                        if let Some(direction) = opts.order_direction {
                            sql.push_str(match direction {
                                OrderDirection::ASC => " ASC",
                                OrderDirection::DESC => " DESC",
                            });
                        }
                    }
                    if let Some(limit) = opts.limit {
                        sql.push_str(&format!(" LIMIT {}", limit));
                    }
                }

                db.query(&sql)
                    .bind(("table", table))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.read_all(table, options).await,
        }
    }

    async fn read_by_ids<T>(
        &self, table: &str, ids: Vec<DbId>, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_things: Vec<Thing> = ids.into_iter().map(Thing::from).collect();
                let mut sql =
                    String::from("SELECT * FROM type::table($table) WHERE id INSIDE $ids");

                if let Some(opts) = options {
                    if let Some(order_by) = opts.order_by {
                        sql.push_str(&format!(" ORDER BY {}", order_by));
                        if let Some(direction) = opts.order_direction {
                            sql.push_str(match direction {
                                OrderDirection::ASC => " ASC",
                                OrderDirection::DESC => " DESC",
                            });
                        }
                    }
                    if let Some(limit) = opts.limit {
                        sql.push_str(&format!(" LIMIT {}", limit));
                    }
                }

                db.query(&sql)
                    .bind(("table", table))
                    .bind(("ids", surreal_things))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.read_by_ids(table, ids, options).await,
        }
    }

    async fn read_by_field<T>(
        &self, table: &str, field: &str, value: &str, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let mut sql = format!("SELECT * FROM type::table($table) WHERE {} = $value", field);

                if let Some(opts) = options {
                    if let Some(order_by) = opts.order_by {
                        sql.push_str(&format!(" ORDER BY {}", order_by));
                        if let Some(direction) = opts.order_direction {
                            sql.push_str(match direction {
                                OrderDirection::ASC => " ASC",
                                OrderDirection::DESC => " DESC",
                            });
                        }
                    }
                    if let Some(limit) = opts.limit {
                        sql.push_str(&format!(" LIMIT {}", limit));
                    }
                }

                db.query(&sql)
                    .bind(("table", table))
                    .bind(("value", value))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.read_by_field(table, field, value, options).await,
        }
    }

    async fn read_by_field_thing<T>(
        &self, table: &str, field: &str, value: DbId, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(value);
                let mut sql = format!(
                    "SELECT * FROM type::table($table) WHERE {} = type::thing($value)",
                    field
                );

                if let Some(opts) = options {
                    if let Some(order_by) = opts.order_by {
                        sql.push_str(&format!(" ORDER BY {}", order_by));
                        if let Some(direction) = opts.order_direction {
                            sql.push_str(match direction {
                                OrderDirection::ASC => " ASC",
                                OrderDirection::DESC => " DESC",
                            });
                        }
                    }
                    if let Some(limit) = opts.limit {
                        sql.push_str(&format!(" LIMIT {}", limit));
                    }
                }

                db.query(&sql)
                    .bind(("table", table))
                    .bind(("value", surreal_thing))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.read_by_field_thing(table, field, value, options).await,
        }
    }

    async fn update_field<T>(&self, thing: DbId, field: &str, value: T) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                db.update(surreal_thing)
                    .patch(PatchOp::replace(field, value))
                    .await
                    .map(|_: Option<serde_json::Value>| ())
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.update_field(thing, field, value).await,
        }
    }

    async fn add_to_field<T>(&self, thing: DbId, field: &str, value: T) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                db.update(surreal_thing)
                    .patch(PatchOp::add(field, value))
                    .await
                    .map(|_: Option<serde_json::Value>| ())
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.add_to_field(thing, field, value).await,
        }
    }

    async fn remove_from_field<T>(
        &self, thing: DbId, field: &str, value: T,
    ) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + std::fmt::Display + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                let sql = "LET $array = (SELECT VALUE type::fields([$field]) FROM ONLY type::thing($id))[0];
                          LET $pos = array::find_index($array, type::thing($pos));
                          LET $value = array::remove($array, $pos);
                          UPDATE type::thing($id) PATCH [{ 'op': 'replace', 'path' : $field, 'value': $value }];";

                db.query(sql)
                    .bind(("id", surreal_thing))
                    .bind(("field", field))
                    .bind(("pos", value.to_string()))
                    .await?;

                Ok(())
            }
            Self::Mock(db) => db.remove_from_field(thing, field, value).await,
        }
    }

    async fn check_organization_access(
        &self, user_id: DbId, organization_id: DbId, required_role: OrganizationRole,
    ) -> Result<bool, DatabaseError> {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let sql = match required_role {
                    OrganizationRole::Owner => {
                        "SELECT * FROM type::thing($org_id)
                        WHERE activated = true
                        AND owner = type::thing($user_id)"
                    }
                    OrganizationRole::Moderator => {
                        "SELECT * FROM type::thing($org_id)
                        WHERE activated = true
                        AND (owner = type::thing($user_id)
                        OR type::thing($user_id) INSIDE moderators)"
                    }
                    OrganizationRole::User => {
                        "SELECT * FROM type::thing($org_id)
                        WHERE activated = true
                        AND (owner = type::thing($user_id)
                        OR type::thing($user_id) INSIDE moderators
                        OR type::thing($user_id) INSIDE users)"
                    }
                };

                let surreal_org = Thing::from(organization_id);
                let surreal_user = Thing::from(user_id);

                let result: Vec<serde_json::Value> = db
                    .query(sql)
                    .bind(("org_id", surreal_org))
                    .bind(("user_id", surreal_user))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)?;

                Ok(!result.is_empty())
            }
            Self::Mock(db) => {
                db.check_organization_access(user_id, organization_id, required_role)
                    .await
            }
        }
    }

    async fn delete_soft(&self, thing: DbId) -> Result<(), DatabaseError> {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let surreal_thing = Thing::from(thing);
                db.update(surreal_thing)
                    .patch(PatchOp::replace("/activated", false))
                    .await
                    .map(|_: Option<serde_json::Value>| ())
                    .map_err(DatabaseError::from)
            }
            Self::Mock(db) => db.delete_soft(thing).await,
        }
    }

    async fn live<T>(&self, table: &str) -> Result<DatabaseStream<Vec<T>>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let db = Arc::new(db.clone());
                let (tx, rx) = mpsc::channel(100);
                let table = table.to_string();

                tokio::spawn(async move {
                    let mut stream = db
                        .select(table)
                        .live()
                        .await
                        .expect("Failed to create live stream");
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(notification) => {
                                let data = match notification.action {
                                    surrealdb::Action::Create | surrealdb::Action::Update => {
                                        match serde_json::from_value::<T>(notification.data) {
                                            Ok(value) => vec![value],
                                            Err(_) => Vec::new(),
                                        }
                                    }
                                    surrealdb::Action::Delete => Vec::new(),
                                    _ => Vec::new(),
                                };
                                if tx.send(Ok(data)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                if tx.send(Err(DatabaseError::from(e))).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                });

                Ok(Box::pin(ReceiverStream::new(rx)))
            }
            Self::Mock(db) => db.live(table).await,
        }
    }

    async fn handle_actions<D>(
        &self, notification: DatabaseNotification<D>, tx: Sender<Option<Vec<D>>>, id: Option<D::Id>,
    ) -> Result<(), DatabaseError>
    where
        D: HasId + SerdeSerialize + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(_db) => {
                let data = notification.data.clone();

                match notification.action {
                    DatabaseAction::Create | DatabaseAction::Update => {
                        if let Some(ref id) = id {
                            if id == data.id() {
                                let _ = tx.send(Some(vec![data]));
                            }
                        } else {
                            let _ = tx.send(Some(vec![data]));
                        }
                    }
                    DatabaseAction::Delete => {
                        if let Some(ref id) = id {
                            if id == data.id() {
                                tx.send(None).unwrap();
                            }
                        } else {
                            tx.send(None).unwrap();
                        }
                    }
                }

                Ok(())
            }
            Self::Mock(db) => db.handle_actions(notification, tx, id).await,
        }
    }

    async fn renew_session<T>(&self, session_key: String) -> Result<T, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + Clone + 'static,
    {
        match self {
            #[cfg(feature = "surrealdb")]
            Self::Surreal(db) => {
                let ttl = Datetime::from(Utc::now() + chrono::Duration::days(2));
                let result: Vec<T> = db
                    .query("UPDATE sessions SET expires_at = $ttl WHERE data = $session_key;")
                    .bind(("ttl", ttl))
                    .bind(("session_key", session_key))
                    .await?
                    .take(0)
                    .map_err(DatabaseError::from)?;

                Result::Ok(result[0].clone())
            }
            Self::Mock(db) => db.renew_session(session_key).await,
        }
    }
}

#[allow(dead_code)]
#[automock(type DatabaseError = crate::error::DatabaseError;)]
#[async_trait]
pub trait DatabaseOperations: Send + Sync + 'static {
    async fn create<T, U>(&self, table: &str, data: T) -> Result<Vec<U>, DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static,
        U: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn select<T>(&self, thing: DbId) -> Result<Option<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn query<U>(
        &self, sql: &str, bindings: Option<QueryBindings>,
    ) -> Result<Vec<U>, DatabaseError>
    where
        U: for<'de> SerdeDeserialize<'de> + Send + Sync + Clone + 'static;

    async fn update<T>(&self, thing: DbId, data: T) -> Result<Option<T>, DatabaseError>
    where
        T: SerdeSerialize + for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn delete(&self, thing: DbId) -> Result<Option<()>, DatabaseError>;

    async fn read_all<T>(
        &self, table: &str, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn read_by_ids<T>(
        &self, table: &str, ids: Vec<DbId>, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn read_by_field<T>(
        &self, table: &str, field: &str, value: &str, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn read_by_field_thing<T>(
        &self, table: &str, field: &str, value: DbId, options: Option<QueryOptions>,
    ) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn update_field<T>(
        &self, thing: DbId, field: &str, value: T,
    ) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static;

    async fn add_to_field<T>(
        &self, thing: DbId, field: &str, value: T,
    ) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + 'static;

    async fn remove_from_field<T>(
        &self, thing: DbId, field: &str, value: T,
    ) -> Result<(), DatabaseError>
    where
        T: SerdeSerialize + Send + Sync + std::fmt::Display + 'static;

    async fn check_organization_access(
        &self, user_id: DbId, organization_id: DbId, required_role: OrganizationRole,
    ) -> Result<bool, DatabaseError>;

    async fn delete_soft(&self, thing: DbId) -> Result<(), DatabaseError>;

    async fn live<T>(&self, table: &str) -> Result<DatabaseStream<Vec<T>>, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + 'static;

    async fn handle_actions<D>(
        &self, notification: DatabaseNotification<D>, tx: Sender<Option<Vec<D>>>, id: Option<D::Id>,
    ) -> Result<(), DatabaseError>
    where
        D: HasId + SerdeSerialize + Send + Sync + 'static;

    async fn renew_session<T>(&self, session_key: String) -> Result<T, DatabaseError>
    where
        T: for<'de> SerdeDeserialize<'de> + HasId + Send + Sync + Clone + 'static;
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DbIdWrapper(pub DbId);

impl Display for DbIdWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SomeStruct {
    id: DbIdWrapper,
    tb: String,
}

impl HasId for SomeStruct {
    type Id = DbIdWrapper;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Clone for MockDatabaseOperations {
    fn clone(&self) -> Self {
        MockDatabaseOperations::new()
    }
}
