// utils/env.rs
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

use std::{collections::HashMap, env};

/// Returns a HashMap containing all environment variables.
///
/// # Examples
///
/// ```rust
/// use kiro_database::get_envv;
/// let envv = get_envv();
/// println!("Environment variables: {:?}", envv);
/// ```
pub fn get_envv() -> HashMap<String, String> {
    env::vars().collect()
}

/// Returns the value of an environment variable or a default value if not found.
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - The default value to return if variable is not found
///
/// # Examples
///
/// ```rust
/// let value = kiro_database::get_env_or("DATABASE_URL", "postgres://localhost/db");
/// ```
pub fn get_env_or(key: &str, default: &str) -> String {
    let envv = get_envv();
    if envv.contains_key(key) {
        // Safety: The HashMap is already checked for the key
        envv.get(key).unwrap().clone()
    } else {
        default.to_string()
    }
}

/// Returns the value of an environment variable, panicking if not found.
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
///
/// # Panics
///
/// Panics if the specified environment variable is not set.
///
/// # Examples
///
/// ```rust,should_panic
/// use kiro_database::get_env_unsafe;
/// let database_url = get_env_unsafe("DATABASE_URL");
/// ```
pub fn get_env_unsafe(key: &str) -> String {
    let envv = get_envv();
    if envv.contains_key(key) {
        envv.get(key).unwrap().clone()
    } else {
        panic!("Enviroment variable {0} is not set", key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_envv() {
        let envv = get_envv();
        assert!(envv.contains_key("PATH"));
    }

    #[test]
    fn test_get_env_or() {
        let envv = get_env_or("PATH", "test");
        assert_eq!(envv, env::var("PATH").unwrap());
    }

    #[test]
    fn test_get_env_unsafe() {
        let envv = get_env_unsafe("PATH");
        assert_eq!(envv, env::var("PATH").unwrap());
    }
}
