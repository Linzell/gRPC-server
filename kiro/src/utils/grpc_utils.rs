// utils/grpc_utils.rs
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

use tonic::metadata::MetadataMap;

/// # Get token from metadata
///
/// Extracts a Bearer token from the gRPC metadata.
///
/// ## Arguments
///
/// * `md` - The gRPC metadata map containing the authorization header
///
/// ## Returns
///
/// * `Ok(String)` - The extracted token without the "Bearer " prefix
/// * `Err(Error)` - If the token is missing, malformed, or empty
///
/// ## Examples
///
/// ```
/// use tonic::metadata::{MetadataMap, MetadataKey, MetadataValue};
/// use std::str::FromStr;
///
/// let mut md = MetadataMap::new();
/// let key = MetadataKey::from_bytes(b"authorization").unwrap();
/// let value = MetadataValue::from_str("Bearer my_token").unwrap();
/// md.insert(key, value);
///
/// let token = get_token_from_md(&md).unwrap();
/// assert_eq!(token, "my_token");
/// ```
pub fn get_token_from_md(md: &MetadataMap) -> Result<String, kiro_database::DatabaseError> {
    let token_string = md.get("authorization").ok_or_else(|| {
        kiro_database::DatabaseError::Internal("Authorization flow error".to_string())
    })?;

    let token_str = token_string.to_str().map_err(|_| {
        kiro_database::DatabaseError::Internal("Authorization flow error".to_string())
    })?;

    if let Some(token) = token_str.strip_prefix("Bearer ") {
        let token = token.trim();
        if token.is_empty() {
            return Err(kiro_database::DatabaseError::Internal(
                "Empty token".to_string(),
            ));
        }
        Ok(token.to_string())
    } else {
        Err(kiro_database::DatabaseError::Internal(
            "Invalid token format".to_string(),
        ))
    }
}

/// # Get IP from metadata
///
/// Extracts the IP address from the x-forwarded-for header in the gRPC metadata.
///
/// ## Arguments
///
/// * `metadata` - The gRPC metadata map containing the x-forwarded-for header
///
/// ## Returns
///
/// * `Some(String)` - The extracted IP address
/// * `None` - If the header is missing or malformed
///
/// ## Examples
///
/// ```
/// use tonic::metadata::{MetadataMap, MetadataKey, MetadataValue};
/// use std::str::FromStr;
///
/// let mut md = MetadataMap::new();
/// md.insert("x-forwarded-for", MetadataValue::from_str("127.0.0.1").unwrap());
///
/// let ip = get_ip_from_md(&md);
/// assert_eq!(ip, Some("127.0.0.1".to_string()));
/// ```
pub fn get_ip_from_md(metadata: &MetadataMap) -> Option<String> {
    metadata
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tonic::metadata::{MetadataKey, MetadataValue};

    #[test]
    fn test_get_token_from_md() {
        let mut md = MetadataMap::new();
        let key = MetadataKey::from_bytes(b"authorization").unwrap();
        let value = MetadataValue::from_str("Bearer token").unwrap();
        md.insert(key, value);

        assert_eq!(get_token_from_md(&md).unwrap(), "token".to_string());
    }

    #[test]
    fn test_get_token_from_md_no_token() {
        let md = MetadataMap::new();
        assert!(get_token_from_md(&md).is_err());
    }

    #[test]
    fn test_get_token_from_md_no_bearer() {
        let mut md = MetadataMap::new();
        let key = MetadataKey::from_bytes(b"authorization").unwrap();
        let value = MetadataValue::from_str("token").unwrap();
        md.insert(key, value);

        assert!(get_token_from_md(&md).is_err());
    }

    #[test]
    fn test_get_token_from_md_empty_token() {
        let mut md = MetadataMap::new();
        let key = MetadataKey::from_bytes(b"authorization").unwrap();
        let value = MetadataValue::from_str("Bearer ").unwrap();
        md.insert(key, value);

        assert!(get_token_from_md(&md).is_err());
    }

    #[test]
    fn test_get_ip_from_md() {
        let mut md = MetadataMap::new();
        let value = MetadataValue::from_str("127.0.0.1").unwrap();
        md.insert("x-forwarded-for", value);

        assert_eq!(get_ip_from_md(&md), Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_get_ip_from_md_missing() {
        let md = MetadataMap::new();
        assert_eq!(get_ip_from_md(&md), None);
    }
}
