// config.rs

use std::fmt;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::Path;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, fs};

use serde::{Deserialize, Serialize};

use crate::utils::{env::get_env_or, error::Error};

lazy_static! {
    static ref CONFIG: Mutex<Arc<Configuration>> = Mutex::new(Arc::new(Default::default()));
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Configuration {
    pub logging: Logging,
    // #[cfg(feature = "postgres")]
    // pub postgresql: Postgresql,
    // #[cfg(feature = "surrealdb")]
    // pub surrealdb: Surreal,
    // pub redis: Redis,
    pub api: Api,
    pub gateway: Gateway,
    // pub network: Network,
    pub monitoring: Monitoring,
    // pub integration: Integration,
    // pub codec: Codec,
    pub user_authentication: UserAuthentication,
    pub join_server: JoinServer,
    pub backend_interfaces: BackendInterfaces,
    // pub roaming: Roaming,
    // pub keks: Vec<Kek>,
    // pub regions: Vec<Region>,
    // pub ui: UI,
}

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {}", self.message)
    }
}

impl std::error::Error for ConfigError {}

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        ConfigError {
            message: message.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Logging {
    pub level: String,
    pub json: bool,
}

impl Default for Logging {
    fn default() -> Self {
        Logging {
            level: "INFO".into(),
            json: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Api {
    pub bind: SocketAddr,
    pub secret: String,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            bind: SocketAddr::new(
                IpAddr::from(Ipv6Addr::UNSPECIFIED),
                get_env_or("PORT", "50051").parse().unwrap(),
            ),
            secret: "".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Gateway {
    #[serde(with = "humantime_serde")]
    pub client_cert_lifetime: Duration,
    pub ca_cert: String,
    pub ca_key: String,
    pub allow_unknown_gateways: bool,
}

impl Default for Gateway {
    fn default() -> Self {
        Gateway {
            client_cert_lifetime: Duration::from_secs(60 * 60 * 24 * 365),
            ca_cert: "".to_string(),
            ca_key: "".to_string(),
            allow_unknown_gateways: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Monitoring {
    pub bind: String,
    pub api_request_log_max_history: usize,
    pub backend_interfaces_log_max_history: usize,
    pub meta_log_max_history: usize,
    pub gateway_frame_log_max_history: usize,
    pub device_frame_log_max_history: usize,
    pub device_event_log_max_history: usize,
    pub per_gateway_frame_log_max_history: usize,
    #[serde(with = "humantime_serde")]
    pub per_gateway_frame_log_ttl: Duration,
    pub per_device_frame_log_max_history: usize,
    #[serde(with = "humantime_serde")]
    pub per_device_frame_log_ttl: Duration,
    pub per_device_event_log_max_history: usize,
    #[serde(with = "humantime_serde")]
    pub per_device_event_log_ttl: Duration,
}

impl Default for Monitoring {
    fn default() -> Self {
        Monitoring {
            bind: "".to_string(),
            api_request_log_max_history: 10,
            backend_interfaces_log_max_history: 10,
            meta_log_max_history: 10,
            gateway_frame_log_max_history: 10,
            device_frame_log_max_history: 10,
            device_event_log_max_history: 10,
            per_gateway_frame_log_max_history: 10,
            per_device_frame_log_max_history: 10,
            per_device_event_log_max_history: 10,
            per_gateway_frame_log_ttl: Duration::from_secs(60 * 60 * 24 * 31),
            per_device_frame_log_ttl: Duration::from_secs(60 * 60 * 24 * 31),
            per_device_event_log_ttl: Duration::from_secs(60 * 60 * 24 * 31),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct UserAuthentication {
    pub enabled: String,
    pub openid_connect: OpenIdConnect,
    pub oauth2: OAuth2,
}

impl Default for UserAuthentication {
    fn default() -> Self {
        UserAuthentication {
            enabled: "internal".into(),
            openid_connect: Default::default(),
            oauth2: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct OpenIdConnect {
    pub registration_enabled: bool,
    pub registration_callback_url: String,
    pub provider_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub logout_url: String,
    pub login_redirect: bool,
    pub login_label: String,
    pub assume_email_verified: bool,
    pub scopes: Vec<String>,
}

impl Default for OpenIdConnect {
    fn default() -> Self {
        OpenIdConnect {
            registration_enabled: false,
            registration_callback_url: "".to_string(),
            provider_url: "".to_string(),
            client_id: "".to_string(),
            client_secret: "".to_string(),
            redirect_url: "".to_string(),
            logout_url: "".to_string(),
            login_redirect: false,
            login_label: "".to_string(),
            assume_email_verified: false,
            scopes: vec!["email".to_string(), "profile".to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct OAuth2 {
    pub registration_enabled: bool,
    pub registration_callback_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub userinfo_url: String,
    pub provider: String,
    pub logout_url: String,
    pub login_redirect: bool,
    pub login_label: String,
    pub assume_email_verified: bool,
    pub scopes: Vec<String>,
}

impl Default for OAuth2 {
    fn default() -> Self {
        OAuth2 {
            registration_enabled: false,
            registration_callback_url: "".to_string(),
            client_id: "".to_string(),
            client_secret: "".to_string(),
            auth_url: "".to_string(),
            token_url: "".to_string(),
            redirect_url: "".to_string(),
            userinfo_url: "".to_string(),
            provider: "".to_string(),
            logout_url: "".to_string(),
            login_redirect: false,
            login_label: "".to_string(),
            assume_email_verified: false,
            scopes: vec!["email".to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct JoinServer {
    pub servers: Vec<JoinServerServer>,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct JoinServerServer {
    // #[serde(alias = "join_eui")]
    // pub join_eui_prefix: EUI64Prefix,
    pub server: String,
    #[serde(with = "humantime_serde")]
    pub async_timeout: Duration,
    pub ca_cert: String,
    pub tls_cert: String,
    pub tls_key: String,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct BackendInterfaces {
    pub bind: String,
    pub ca_cert: String,
    pub tls_cert: String,
    pub tls_key: String,
}

pub fn load(config_dir: &Path) -> Result<(), Error> {
    let mut content: String = String::new();

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        let default_config = toml::to_string(&Configuration::default())?;
        fs::write(&config_file, default_config)?;
    }

    let paths = fs::read_dir(config_dir)?;
    for path in paths {
        let path = path.unwrap().path();

        if let Some(ext) = path.extension() {
            if ext == "toml" {
                content.push_str(&fs::read_to_string(&path).map_err(|e| {
                    Error::Configuration(ConfigError::new(format!(
                        "could not read configuration file '{}': {}",
                        path.display(),
                        e
                    )))
                })?);
            }
        }
    }

    // substitute environment variables in config file
    for (k, v) in env::vars() {
        content = content.replace(&format!("${}", k), &v);
    }

    let conf: Configuration = toml::from_str(&content)?;
    set(conf);

    Ok(())
}

pub fn set(c: Configuration) {
    let mut conf_mutex = CONFIG.lock().unwrap();
    *conf_mutex = Arc::new(c);
}

pub fn get() -> Arc<Configuration> {
    let conf = CONFIG.lock().unwrap();
    conf.clone()
}

pub fn save(conf: &Configuration, config_dir: &Path) -> Result<(), Error> {
    let config_file = config_dir.join("config.toml");
    let content = toml::to_string_pretty(conf)?;
    fs::write(config_file, content)?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ConfigField {
    pub path: String,
    pub description: String,
    pub possible_values: String,
    pub getter: fn(&Configuration) -> String,
    pub setter: fn(&mut Configuration, &str) -> Result<(), String>,
}

impl Configuration {
    pub fn get_interactive_fields() -> Vec<ConfigField> {
        let mut fields = Vec::new();

        fields.extend(Self::get_logging_fields());
        // fields.extend(Self::get_postgresql_fields());
        // fields.extend(Self::get_redis_fields());
        fields.extend(Self::get_api_fields());
        fields.extend(Self::get_gateway_fields());
        // fields.extend(Self::get_network_fields());
        fields.extend(Self::get_monitoring_fields());
        // fields.extend(Self::get_integration_fields());
        // fields.extend(Self::get_codec_fields());
        fields.extend(Self::get_user_authentication_fields());
        fields.extend(Self::get_join_server_fields());
        fields.extend(Self::get_backend_interfaces_fields());
        // fields.extend(Self::get_roaming_fields());
        // fields.extend(Self::get_ui_fields());

        fields
    }

    fn get_logging_fields() -> Vec<ConfigField> {
        vec![
            ConfigField {
                path: "logging.level".to_string(),
                description: "Log level".to_string(),
                possible_values: "TRACE, DEBUG, INFO, WARN, ERROR, OFF".to_string(),
                getter: |conf| conf.logging.level.clone(),
                setter: |conf, value| {
                    let upper_value = value.to_uppercase();
                    if ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"]
                        .contains(&upper_value.as_str())
                    {
                        conf.logging.level = upper_value;
                        Ok(())
                    } else {
                        Err("Invalid log level".to_string())
                    }
                },
            },
            ConfigField {
                path: "logging.json".to_string(),
                description: "Log as JSON".to_string(),
                possible_values: "true, false".to_string(),
                getter: |conf| conf.logging.json.to_string(),
                setter: |conf, value| {
                    conf.logging.json = value
                        .parse()
                        .map_err(|_| "Invalid boolean value".to_string())?;
                    Ok(())
                },
            },
        ]
    }

    // fn get_postgresql_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "postgresql.dsn".to_string(),
    //             description: "PostgreSQL DSN".to_string(),
    //             possible_values: "postgres://<USERNAME>:<PASSWORD>@<HOSTNAME>/<DATABASE>?sslmode=<SSLMODE>".to_string(),
    //             getter: |conf| conf.postgresql.dsn.clone(),
    //             setter: |conf, value| {
    //                 conf.postgresql.dsn = value;
    //                 Ok(())
    //             },
    //         },
    //         ConfigField {
    //             path: "postgresql.max_open_connections".to_string(),
    //             description: "Max open connections".to_string(),
    //             possible_values: "Positive integer".to_string(),
    //             getter: |conf| conf.postgresql.max_open_connections.to_string(),
    //             setter: |conf, value| {
    //                 conf.postgresql.max_open_connections = value.parse().map_err(|_| "Invalid integer".to_string())?;
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }

    // fn get_redis_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "redis.servers".to_string(),
    //             description: "Redis server addresses".to_string(),
    //             possible_values: "Comma-separated list of redis://host:port".to_string(),
    //             getter: |conf| conf.redis.servers.join(", "),
    //             setter: |conf, value| {
    //                 conf.redis.servers = value.split(',').map(|s| s.trim().to_string()).collect();
    //                 Ok(())
    //             },
    //         },
    //         ConfigField {
    //             path: "redis.cluster".to_string(),
    //             description: "Redis Cluster".to_string(),
    //             possible_values: "true, false".to_string(),
    //             getter: |conf| conf.redis.cluster.to_string(),
    //             setter: |conf, value| {
    //                 conf.redis.cluster = value.parse().map_err(|_| "Invalid boolean value".to_string())?;
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }

    fn get_api_fields() -> Vec<ConfigField> {
        vec![
            ConfigField {
                path: "api.bind".to_string(),
                description: "API bind address".to_string(),
                possible_values: "<IP>:<PORT> (e.g., 127.0.0.1:8080)".to_string(),
                getter: |conf| conf.api.bind.to_string(),
                setter: |conf, value| {
                    conf.api.bind = value
                        .parse()
                        .map_err(|_| "Invalid socket address".to_string())?;
                    Ok(())
                },
            },
            ConfigField {
                path: "api.secret".to_string(),
                description: "API secret".to_string(),
                possible_values: "Strong secret string".to_string(),
                getter: |conf| conf.api.secret.clone(),
                setter: |conf, value| {
                    conf.api.secret = value.to_string();
                    Ok(())
                },
            },
        ]
    }

    fn get_gateway_fields() -> Vec<ConfigField> {
        vec![
            ConfigField {
                path: "gateway.ca_cert".to_string(),
                description: "CA certificate path".to_string(),
                possible_values: "Path to CA certificate file".to_string(),
                getter: |conf| conf.gateway.ca_cert.clone(),
                setter: |conf, value| {
                    conf.gateway.ca_cert = value.to_string();
                    Ok(())
                },
            },
            ConfigField {
                path: "gateway.ca_key".to_string(),
                description: "CA key path".to_string(),
                possible_values: "Path to CA key file".to_string(),
                getter: |conf| conf.gateway.ca_key.clone(),
                setter: |conf, value| {
                    conf.gateway.ca_key = value.to_string();
                    Ok(())
                },
            },
        ]
    }

    // fn get_network_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "network.net_id".to_string(),
    //             description: "Network identifier (NetID)".to_string(),
    //             possible_values: "3 bytes encoded as HEX (e.g. 010203)".to_string(),
    //             getter: |conf| conf.network.net_id.clone(),
    //             setter: |conf, value| {
    //                 if value.len() == 6 && value.chars().all(|c| c.is_digit(16)) {
    //                     conf.network.net_id = value;
    //                     Ok(())
    //                 } else {
    //                     Err("Invalid NetID format".to_string())
    //                 }
    //             },
    //         },
    //     ]
    // }

    fn get_monitoring_fields() -> Vec<ConfigField> {
        vec![ConfigField {
            path: "monitoring.bind".to_string(),
            description: "Monitoring bind address".to_string(),
            possible_values: "<IP>:<PORT> (e.g., 127.0.0.1:8080)".to_string(),
            getter: |conf| conf.monitoring.bind.clone(),
            setter: |conf, value| {
                conf.monitoring.bind = value.to_string();
                Ok(())
            },
        }]
    }

    // fn get_integration_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "integration.enabled".to_string(),
    //             description: "Enabled integrations".to_string(),
    //             possible_values: "Comma-separated list of integrations".to_string(),
    //             getter: |conf| conf.integration.enabled.join(", "),
    //             setter: |conf, value| {
    //                 conf.integration.enabled = value.split(',').map(|s| s.trim().to_string()).collect();
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }

    // fn get_codec_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "codec.js.max_execution_time".to_string(),
    //             description: "JS codec max execution time".to_string(),
    //             possible_values: "Duration (e.g., 100ms, 5s)".to_string(),
    //             getter: |conf| conf.codec.js.max_execution_time.clone(),
    //             setter: |conf, value| {
    //                 conf.codec.js.max_execution_time = value;
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }

    fn get_user_authentication_fields() -> Vec<ConfigField> {
        vec![ConfigField {
            path: "user_authentication.enabled".to_string(),
            description: "Enabled authentication backend".to_string(),
            possible_values: "internal, openid_connect, oauth2".to_string(),
            getter: |conf| conf.user_authentication.enabled.clone(),
            setter: |conf, value| {
                if ["internal", "openid_connect", "oauth2"].contains(&value) {
                    conf.user_authentication.enabled = value.to_string();
                    Ok(())
                } else {
                    Err("Invalid authentication backend".to_string())
                }
            },
        }]
    }

    fn get_join_server_fields() -> Vec<ConfigField> {
        vec![
            // Add fields for join_server configuration if needed
        ]
    }

    fn get_backend_interfaces_fields() -> Vec<ConfigField> {
        vec![ConfigField {
            path: "backend_interfaces.bind".to_string(),
            description: "Backend Interfaces bind address".to_string(),
            possible_values: "<IP>:<PORT> (e.g., 127.0.0.1:8080)".to_string(),
            getter: |conf| conf.backend_interfaces.bind.clone(),
            setter: |conf, value| {
                conf.backend_interfaces.bind = value.to_string();
                Ok(())
            },
        }]
    }

    // fn get_roaming_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "roaming.resolve_net_id_domain_suffix".to_string(),
    //             description: "Resolve NetID domain suffix".to_string(),
    //             possible_values: "Domain suffix".to_string(),
    //             getter: |conf| conf.roaming.resolve_net_id_domain_suffix.clone(),
    //             setter: |conf, value| {
    //                 conf.roaming.resolve_net_id_domain_suffix = value;
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }

    // fn get_ui_fields() -> Vec<ConfigField> {
    //     vec![
    //         ConfigField {
    //             path: "ui.tileserver_url".to_string(),
    //             description: "Tileserver URL".to_string(),
    //             possible_values: "URL of the tileserver".to_string(),
    //             getter: |conf| conf.ui.tileserver_url.clone(),
    //             setter: |conf, value| {
    //                 conf.ui.tileserver_url = value;
    //                 Ok(())
    //             },
    //         },
    //         ConfigField {
    //             path: "ui.map_attribution".to_string(),
    //             description: "Map attribution".to_string(),
    //             possible_values: "Attribution text for the map".to_string(),
    //             getter: |conf| conf.ui.map_attribution.clone(),
    //             setter: |conf, value| {
    //                 conf.ui.map_attribution = value;
    //                 Ok(())
    //             },
    //         },
    //     ]
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();

        load(config_dir).unwrap();

        let loaded_config = get();
        assert_eq!(loaded_config.logging.level, "INFO");
        assert_eq!(loaded_config.logging.json, false);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();

        let mut custom_config = Configuration::default();
        custom_config.logging.level = "debug".to_string();
        custom_config.api.secret = "test_secret".to_string();

        save(&custom_config, config_dir).unwrap();
        load(config_dir).unwrap();

        let loaded_config = get();
        assert_eq!(loaded_config.logging.level, "debug");
        assert_eq!(loaded_config.api.secret, "test_secret");
    }

    #[test]
    fn test_set_and_get_config() {
        let mut custom_config = Configuration::default();
        custom_config.logging.json = true;
        custom_config.api.bind = "127.0.0.1:9000".parse().unwrap();

        set(custom_config.clone());

        let retrieved_config = get();
        assert_eq!(retrieved_config.logging.json, true);
        assert_eq!(retrieved_config.api.bind.to_string(), "127.0.0.1:9000");
    }

    #[test]
    fn test_get_interactive_fields() {
        let fields = Configuration::get_interactive_fields();

        // Check if some expected fields are present
        assert!(fields.iter().any(|f| f.path == "logging.level"));
        assert!(fields.iter().any(|f| f.path == "api.bind"));
        assert!(fields.iter().any(|f| f.path == "gateway.ca_cert"));
    }

    #[test]
    fn test_config_field_setter() {
        let mut config = Configuration::default();
        let fields = Configuration::get_interactive_fields();

        // Test setting a valid log level
        let log_level_field = fields.iter().find(|f| f.path == "logging.level").unwrap();
        (log_level_field.setter)(&mut config, "DEBUG").unwrap();
        assert_eq!(config.logging.level, "DEBUG");

        // Test setting an invalid log level
        assert!((log_level_field.setter)(&mut config, "INVALID").is_err());

        // Test setting a valid boolean
        let json_logging_field = fields.iter().find(|f| f.path == "logging.json").unwrap();
        (json_logging_field.setter)(&mut config, "true").unwrap();
        assert_eq!(config.logging.json, true);

        // Test setting an invalid boolean
        assert!((json_logging_field.setter)(&mut config, "not_a_bool").is_err());
    }
}
