// utils/env.rs

use std::{collections::HashMap, env};

/// # Get envv
///
/// The `get_envv` method returns a HashMap of environment variables.
///
/// ```rust
/// let envv = get_envv();
///
/// println!("🔧 Enviroment variables: {
///  #envv
/// }");
/// ```
fn get_envv() -> HashMap<String, String> {
    env::vars().collect()
}

/// # Get env or
///
/// The `get_env_or` method returns either the value of an environment variable or the default provided.
///
/// ```rust
/// let value = get_env_or("KEY", "default");
///
/// println!("🔑 Value: {:?}", value);
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

/// # Get env unsafe
///
/// The `get_env_unsafe` method returns the value of an environment variable.
///
/// **<!> Note: The method panics if the variable does not exist.**
///
/// ```rust
/// let value = get_env_unsafe("KEY");
///
/// println!("🔑 Value: {:?}", value);
/// ```
pub fn get_env_unsafe(key: &str) -> String {
    let envv = get_envv();
    if envv.contains_key(key) {
        envv.get(key).unwrap().clone()
    } else {
        panic!("Enviroment variable {0} is not set", key)
    }
}
