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

use std::net::IpAddr;

/// Extracts IP address from request metadata
///
/// # Arguments
/// * `metadata` - Request metadata containing client information
///
/// # Returns
/// * `Option<String>` - IP address if found, None otherwise
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

/// Extracts IP address from request http1 headers
///
/// # Arguments
/// * `headers` - Request headers containing client information
///
/// # Returns
/// * `Option<String>` - IP address if found, None otherwise
pub fn get_ip_from_headers(headers: &http::HeaderMap) -> Option<String> {
    // Helper function to validate IP address
    fn validate_ip(ip_str: &str) -> Option<String> {
        ip_str
            .parse::<IpAddr>()
            .ok()
            .filter(|ip| !ip.is_loopback() && !ip.is_unspecified())
            .map(|ip| ip.to_string())
    }

    // Helper function to extract first IP from comma-separated list
    fn extract_first_ip(header_value: &str) -> Option<String> {
        header_value
            .split(',')
            .next()
            .map(str::trim)
            .and_then(validate_ip)
    }

    // Try X-Forwarded-For header (most common for proxied requests)
    if let Some(forwarded) = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()) {
        if let Some(ip) = extract_first_ip(forwarded) {
            return Some(ip);
        }
    }

    // Try X-Real-IP header (common in Nginx)
    if let Some(real_ip) = headers.get("x-real-ip").and_then(|h| h.to_str().ok()) {
        if let Some(ip) = validate_ip(real_ip.trim()) {
            return Some(ip);
        }
    }

    // Try True-Client-IP header (Cloudflare and some CDNs)
    if let Some(true_ip) = headers.get("true-client-ip").and_then(|h| h.to_str().ok()) {
        if let Some(ip) = validate_ip(true_ip.trim()) {
            return Some(ip);
        }
    }

    // Try CF-Connecting-IP header (Cloudflare specific)
    if let Some(cf_ip) = headers
        .get("cf-connecting-ip")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(ip) = validate_ip(cf_ip.trim()) {
            return Some(ip);
        }
    }

    // Fallback to X-Peer-Addr header
    headers
        .get("x-peer-addr")
        .and_then(|h| h.to_str().ok())
        .and_then(|addr| addr.split(':').next())
        .map(str::trim)
        .and_then(validate_ip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
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

    #[test]
    fn test_valid_ipv4() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.195".parse().unwrap());
        assert_eq!(
            get_ip_from_headers(&headers),
            Some("203.0.113.195".to_string())
        );
    }

    #[test]
    fn test_valid_ipv6() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "2001:db8:85a3:8d3:1319:8a2e:370:7348".parse().unwrap(),
        );
        assert_eq!(
            get_ip_from_headers(&headers),
            Some("2001:db8:85a3:8d3:1319:8a2e:370:7348".to_string())
        );
    }

    #[test]
    fn test_invalid_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "invalid-ip".parse().unwrap());
        assert_eq!(get_ip_from_headers(&headers), None);
    }

    #[test]
    fn test_multiple_ips() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.195, 70.41.3.18, 150.172.238.178"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            get_ip_from_headers(&headers),
            Some("203.0.113.195".to_string())
        );
    }
}
