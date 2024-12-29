// database/db_types.rs
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

use chrono::{DateTime, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DbId {
    pub tb: String,
    pub id: DbIdentifier,
}

// WARNING: This is a default implementation for testing purposes only
impl Default for DbId {
    fn default() -> Self {
        Self {
            tb: "default".to_string(),
            id: DbIdentifier::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum DbIdentifier {
    String(String),
    Number(i64),
    Array(Vec<DbIdentifier>),
    Object(serde_json::Map<String, serde_json::Value>),
    Generate(String),
}

// WARNING: This is a default implementation for testing purposes only
impl Default for DbIdentifier {
    fn default() -> Self {
        DbIdentifier::String("123".to_string())
    }
}

impl PartialOrd for DbIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DbIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (DbIdentifier::String(a), DbIdentifier::String(b)) => a.cmp(b),
            (DbIdentifier::Number(a), DbIdentifier::Number(b)) => a.cmp(b),
            (DbIdentifier::Array(a), DbIdentifier::Array(b)) => a.cmp(b),
            (DbIdentifier::Object(a), DbIdentifier::Object(b)) => {
                let a_str = serde_json::to_string(a).unwrap_or_default();
                let b_str = serde_json::to_string(b).unwrap_or_default();
                a_str.cmp(&b_str)
            }
            (DbIdentifier::Generate(a), DbIdentifier::Generate(b)) => a.cmp(b),
            (DbIdentifier::String(_), _) => Ordering::Less,
            (DbIdentifier::Number(_), DbIdentifier::String(_)) => Ordering::Greater,
            (DbIdentifier::Number(_), _) => Ordering::Less,
            (DbIdentifier::Array(_), DbIdentifier::Object(_)) => Ordering::Less,
            (DbIdentifier::Array(_), DbIdentifier::Generate(_)) => Ordering::Less,
            (DbIdentifier::Array(_), _) => Ordering::Greater,
            (DbIdentifier::Object(_), DbIdentifier::Generate(_)) => Ordering::Less,
            (DbIdentifier::Object(_), _) => Ordering::Greater,
            (DbIdentifier::Generate(_), _) => Ordering::Greater,
        }
    }
}

impl Serialize for DbId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "surrealdb")]
        {
            use surrealdb::sql::Thing;
            // Create Thing with raw string ID and let SurrealDB handle the serialization
            let thing = Thing::from(self.clone());
            thing.serialize(serializer)
        }

        #[cfg(not(feature = "surrealdb"))]
        {
            use serde::ser::SerializeStruct;
            let mut state = serializer.serialize_struct("DbId", 2)?;
            state.serialize_field("tb", &self.tb)?;
            state.serialize_field("id", &self.id)?;
            state.end()
        }
    }
}

impl<'de> Deserialize<'de> for DbId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Visitor;

        #[cfg(feature = "surrealdb")]
        {
            use serde::de::MapAccess;

            struct DbIdVisitor;

            impl<'de> Visitor<'de> for DbIdVisitor {
                type Value = DbId;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("string in table:id format or map containing id")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    let parts: Vec<&str> = value.splitn(2, ':').collect();
                    if parts.len() != 2 {
                        return Err(E::custom("expected table:id format"));
                    }
                    Ok(DbId {
                        tb: parts[0].to_string(),
                        id: DbIdentifier::String(parts[1].to_string()),
                    })
                }

                fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                where
                    M: MapAccess<'de>,
                {
                    let mut tb = None;
                    let mut id = None;

                    while let Some(key) = map.next_key::<String>()? {
                        match key.as_str() {
                            "tb" => {
                                tb = Some(map.next_value::<String>()?);
                            }
                            "id" => {
                                let val: serde_json::Value = map.next_value()?;
                                if let Some(id_str) = val.as_str() {
                                    id = Some(DbIdentifier::String(id_str.to_string()));
                                } else if let Some(obj) = val.as_object() {
                                    if let Some(str_val) =
                                        obj.get("String").and_then(|v| v.as_str())
                                    {
                                        id = Some(DbIdentifier::String(str_val.to_string()));
                                    }
                                }
                            }
                            _ => {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                    }

                    match (tb, id) {
                        (Some(tb), Some(id)) => Ok(DbId { tb, id }),
                        _ => Err(M::Error::custom("missing required fields")),
                    }
                }
            }

            deserializer.deserialize_any(DbIdVisitor)
        }

        #[cfg(not(feature = "surrealdb"))]
        {
            use serde::de::MapAccess;

            struct DbIdHelperVisitor;

            impl<'de> Visitor<'de> for DbIdHelperVisitor {
                type Value = DbId;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct DbId")
                }

                fn visit_map<V>(self, mut map: V) -> Result<DbId, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut tb = None;
                    let mut id = None;

                    while let Some(key) = map.next_key::<String>()? {
                        match key.as_str() {
                            "tb" => {
                                if tb.is_some() {
                                    return Err(Error::duplicate_field("tb"));
                                }
                                tb = Some(map.next_value()?);
                            }
                            "id" => {
                                if id.is_some() {
                                    return Err(Error::duplicate_field("id"));
                                }
                                id = Some(map.next_value()?);
                            }
                            _ => {
                                let _: serde::de::IgnoredAny = map.next_value()?;
                            }
                        }
                    }

                    let tb = tb.ok_or_else(|| Error::missing_field("tb"))?;
                    let id = id.ok_or_else(|| Error::missing_field("id"))?;
                    Ok(DbId { tb, id })
                }
            }

            deserializer.deserialize_map(DbIdHelperVisitor)
        }
    }
}

impl<'de> Deserialize<'de> for DbIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[cfg(not(feature = "surrealdb"))]
        use serde::de::Visitor;

        #[cfg(feature = "surrealdb")]
        {
            let value = serde_json::Value::deserialize(deserializer)?;
            match value {
                serde_json::Value::String(s) => Ok(DbIdentifier::String(s)),
                serde_json::Value::Number(n) => Ok(DbIdentifier::Number(n.as_i64().unwrap_or(0))),
                serde_json::Value::Object(obj) => {
                    if let Some(s) = obj.get("String").and_then(|v| v.as_str()) {
                        Ok(DbIdentifier::String(s.to_string()))
                    } else {
                        Ok(DbIdentifier::Object(obj))
                    }
                }
                _ => Ok(DbIdentifier::String("".to_string())),
            }
        }

        #[cfg(not(feature = "surrealdb"))]
        {
            struct DbIdentifierVisitor;

            impl<'de> Visitor<'de> for DbIdentifierVisitor {
                type Value = DbIdentifier;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("string or object value")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(DbIdentifier::String(value.to_owned()))
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(DbIdentifier::String(value))
                }
            }

            deserializer.deserialize_str(DbIdentifierVisitor)
        }
    }
}

impl Serialize for DbIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DbIdentifier::String(s) => s.serialize(serializer),
            DbIdentifier::Number(n) => n.serialize(serializer),
            DbIdentifier::Array(a) => a.serialize(serializer),
            DbIdentifier::Object(o) => o.serialize(serializer),
            DbIdentifier::Generate(s) => s.serialize(serializer),
        }
    }
}

impl From<(&str, DbIdentifier)> for DbId {
    fn from((tb, id): (&str, DbIdentifier)) -> Self {
        Self {
            tb: tb.to_owned(),
            id,
        }
    }
}

impl From<(String, DbIdentifier)> for DbId {
    fn from((tb, id): (String, DbIdentifier)) -> Self {
        Self { tb, id }
    }
}

impl From<(String, String)> for DbId {
    fn from((tb, id): (String, String)) -> Self {
        Self::from((tb, DbIdentifier::String(id)))
    }
}

impl From<(&str, &str)> for DbId {
    fn from((tb, id): (&str, &str)) -> Self {
        Self::from((tb.to_owned(), DbIdentifier::String(id.to_owned())))
    }
}

impl TryFrom<String> for DbId {
    type Error = String;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        Self::try_from(v.as_str())
    }
}

impl TryFrom<&str> for DbId {
    type Error = String;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = v.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Invalid record ID format. Expected table:id".to_string());
        }

        Ok(Self {
            tb: parts[0].to_string(),
            id: DbIdentifier::String(parts[1].to_string()),
        })
    }
}

impl FromStr for DbId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl DbId {
    pub fn to_raw(&self) -> String {
        self.to_string()
    }
}

impl Display for DbId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tb, self.id)
    }
}

impl Display for DbIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbIdentifier::String(s) => write!(f, "{}", s), // Use SurrealDB's record ID format
            DbIdentifier::Number(n) => write!(f, "{}", n),
            DbIdentifier::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", items.join(","))
            }
            DbIdentifier::Object(o) => {
                write!(f, "{}", serde_json::to_string(o).unwrap_or_default())
            }
            DbIdentifier::Generate(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DbDateTime(DateTime<Utc>);

impl Serialize for DbDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "surrealdb")]
        {
            use surrealdb::sql::Datetime;
            let surreal_dt = Datetime::from(self.0);
            surreal_dt.serialize(serializer)
        }

        #[cfg(not(feature = "surrealdb"))]
        {
            self.0.timestamp().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for DbDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[cfg(feature = "surrealdb")]
        {
            use surrealdb::sql::Datetime;
            let dt = Datetime::deserialize(deserializer)?;
            Ok(Self(DateTime::from_timestamp(dt.timestamp(), 0).unwrap()))
        }

        #[cfg(not(feature = "surrealdb"))]
        {
            let timestamp = i64::deserialize(deserializer)?;
            DateTime::from_timestamp(timestamp, 0)
                .map(Self)
                .ok_or_else(|| D::Error::custom("invalid timestamp"))
        }
    }
}

impl From<DateTime<Utc>> for DbDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<DbDateTime> for DateTime<Utc> {
    fn from(dt: DbDateTime) -> Self {
        dt.0
    }
}

impl Display for DbDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DbDateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn inner(&self) -> &DateTime<Utc> {
        &self.0
    }

    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }

    pub fn from_timestamp(secs: i64, nsecs: u32) -> Option<Self> {
        DateTime::from_timestamp(secs, nsecs).map(Self)
    }
}

impl Deref for DbDateTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq<DateTime<Utc>> for DbDateTime {
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        &self.0 == other
    }
}

impl PartialOrd<DateTime<Utc>> for DbDateTime {
    fn partial_cmp(&self, other: &DateTime<Utc>) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

#[cfg(feature = "surrealdb")]
use surrealdb::sql::{Datetime, Id, Object, Thing, Value};

#[cfg(feature = "surrealdb")]
impl From<Id> for DbIdentifier {
    fn from(id: Id) -> Self {
        match id {
            Id::String(s) => DbIdentifier::String(s),
            Id::Number(n) => DbIdentifier::Number(n),
            Id::Array(a) => DbIdentifier::Array(
                a.0.into_iter()
                    .map(|v| match v {
                        Value::Strand(s) => DbIdentifier::String(s.0),
                        Value::Number(n) => DbIdentifier::Number(n.as_int()),
                        Value::Object(o) => {
                            DbIdentifier::Object(convert_surreal_object_to_json_map(o))
                        }
                        _ => DbIdentifier::String("".to_string()),
                    })
                    .collect(),
            ),
            Id::Object(o) => DbIdentifier::Object(convert_surreal_object_to_json_map(o)),
            Id::Generate(_) => match Id::rand() {
                Id::String(s) => DbIdentifier::String(s),
                Id::Number(n) => DbIdentifier::Number(n),
                _ => DbIdentifier::String("".to_string()),
            },
            _ => DbIdentifier::String("".to_string()),
        }
    }
}

#[cfg(feature = "surrealdb")]
impl From<DbIdentifier> for Id {
    fn from(id: DbIdentifier) -> Self {
        match id {
            DbIdentifier::String(s) => Id::String(s),
            DbIdentifier::Number(n) => Id::Number(n),
            DbIdentifier::Array(a) => {
                let values: Vec<Value> = a
                    .into_iter()
                    .map(|v| match v {
                        DbIdentifier::String(s) => Value::from(s),
                        DbIdentifier::Number(n) => Value::from(n),
                        DbIdentifier::Object(o) => {
                            Value::Object(convert_json_map_to_surreal_object(o))
                        }
                        DbIdentifier::Array(_) => Value::None,
                        DbIdentifier::Generate(_) => Value::None,
                    })
                    .collect();
                Id::Array(values.into())
            }
            DbIdentifier::Object(o) => Id::Object(convert_json_map_to_surreal_object(o)),
            DbIdentifier::Generate(s) => match s.as_str() {
                "rand()" => Id::rand(),
                "ulid()" => Id::ulid(),
                "uuid()" => Id::uuid(),
                _ => Id::String(s),
            },
        }
    }
}

#[cfg(feature = "surrealdb")]
fn convert_surreal_object_to_json_map(obj: Object) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in obj.0 {
        map.insert(
            k,
            match v {
                Value::Strand(s) => serde_json::Value::String(s.0),
                Value::Number(n) => {
                    if let Some(num) = serde_json::Number::from_f64(n.as_float()) {
                        serde_json::Value::Number(num)
                    } else {
                        serde_json::Value::Null
                    }
                }
                Value::Bool(b) => serde_json::Value::Bool(b),
                Value::Object(o) => {
                    serde_json::Value::Object(convert_surreal_object_to_json_map(o))
                }
                Value::Array(a) => serde_json::Value::Array(
                    a.0.into_iter().map(surreal_value_to_json_value).collect(),
                ),
                _ => serde_json::Value::Null,
            },
        );
    }
    map
}

#[cfg(feature = "surrealdb")]
fn convert_json_map_to_surreal_object(map: serde_json::Map<String, serde_json::Value>) -> Object {
    let mut obj = Object::default();
    for (k, v) in map {
        obj.0.insert(k, json_value_to_surreal_value(v));
    }
    obj
}

#[cfg(feature = "surrealdb")]
fn surreal_value_to_json_value(value: Value) -> serde_json::Value {
    match value {
        Value::Strand(s) => serde_json::Value::String(s.0),
        Value::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(n.as_float()) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::Null
            }
        }
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Object(o) => serde_json::Value::Object(convert_surreal_object_to_json_map(o)),
        Value::Array(a) => {
            serde_json::Value::Array(a.0.into_iter().map(surreal_value_to_json_value).collect())
        }
        _ => serde_json::Value::Null,
    }
}

#[cfg(feature = "surrealdb")]
fn json_value_to_surreal_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::String(s) => Value::from(s),
        serde_json::Value::Number(n) => {
            let float = n.as_f64().unwrap_or(0.0);
            Value::from(float)
        }
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Object(o) => Value::Object(convert_json_map_to_surreal_object(o)),
        serde_json::Value::Array(a) => {
            let values: Vec<Value> = a.into_iter().map(json_value_to_surreal_value).collect();
            Value::Array(values.into())
        }
        serde_json::Value::Null => Value::None,
    }
}

#[cfg(feature = "surrealdb")]
impl From<Thing> for DbId {
    fn from(thing: Thing) -> Self {
        DbId {
            tb: thing.tb,
            id: DbIdentifier::from(thing.id),
        }
    }
}

#[cfg(feature = "surrealdb")]
impl From<DbId> for Thing {
    fn from(id: DbId) -> Self {
        let raw_id = match id.id {
            DbIdentifier::String(s) => s,
            DbIdentifier::Number(n) => n.to_string(),
            _ => id.id.to_string(),
        };
        Thing::from((id.tb, Id::String(raw_id)))
    }
}

#[cfg(feature = "surrealdb")]
impl From<Datetime> for DbDateTime {
    fn from(dt: Datetime) -> Self {
        Self(DateTime::from_timestamp(dt.timestamp(), 0).unwrap())
    }
}

#[cfg(feature = "surrealdb")]
impl From<DbDateTime> for Datetime {
    fn from(dt: DbDateTime) -> Self {
        Datetime::from(dt.0)
    }
}

impl From<String> for DbIdentifier {
    fn from(s: String) -> Self {
        DbIdentifier::String(s)
    }
}

impl From<&str> for DbIdentifier {
    fn from(s: &str) -> Self {
        DbIdentifier::String(s.to_owned())
    }
}

impl From<i64> for DbIdentifier {
    fn from(n: i64) -> Self {
        DbIdentifier::Number(n)
    }
}
