// utils/ip.rs
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

/// Extracts IP address from request metadata
///
/// # Arguments
/// * `metadata` - Request metadata containing client information
///
/// # Returns
/// * `Option<String>` - IP address if found, None otherwise
///
/// # Example
/// ```
/// let metadata = request.metadata();
/// let ip = get_ip_from_md(metadata).unwrap_or_else(|| "unknown".to_string());
/// ```
pub fn get_ip_from_md(metadata: &tonic::metadata::MetadataMap) -> Option<String> {
    // Try to get IP from X-Forwarded-For header first
    if let Some(forwarded) = metadata.get("x-forwarded-for") {
        if let Ok(ip) = forwarded.to_str() {
            // Get first IP if multiple are present
            return Some(ip.split(',').next()?.trim().to_string());
        }
    }

    // Try to get IP from X-Real-IP header
    if let Some(real_ip) = metadata.get("x-real-ip") {
        if let Ok(ip) = real_ip.to_str() {
            return Some(ip.trim().to_string());
        }
    }

    // Try to get IP from peer addr
    if let Some(peer) = metadata.get("x-peer-addr") {
        if let Ok(addr) = peer.to_str() {
            // Extract IP from socket address (remove port)
            return Some(addr.split(':').next()?.trim().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataMap;

    #[test]
    fn test_get_ip_from_md() {
        let mut metadata = MetadataMap::new();

        // Test X-Forwarded-For
        metadata.insert(
            "x-forwarded-for",
            "203.0.113.195, 70.41.3.18".parse().unwrap(),
        );
        assert_eq!(get_ip_from_md(&metadata).unwrap(), "203.0.113.195");

        // Test X-Real-IP
        metadata.clear();
        metadata.insert("x-real-ip", "192.168.1.1".parse().unwrap());
        assert_eq!(get_ip_from_md(&metadata).unwrap(), "192.168.1.1");

        // Test peer addr
        metadata.clear();
        metadata.insert("x-peer-addr", "10.0.0.1:12345".parse().unwrap());
        assert_eq!(get_ip_from_md(&metadata).unwrap(), "10.0.0.1");

        // Test no IP headers
        metadata.clear();
        assert_eq!(get_ip_from_md(&metadata), None);
    }
}
