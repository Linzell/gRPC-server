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

#[derive(Default, Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Logging {
    pub level: String,
    pub json: bool,
}

impl Default for Logging {
    fn default() -> Self {
        Logging {
            level: "info".into(),
            json: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct JoinServer {
    pub servers: Vec<JoinServerServer>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
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

#[derive(Serialize, Deserialize, Default, Clone)]
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
