use std::{env, net::Ipv4Addr};

use crate::config::Config;

/// # Use this for reading config from Environment Variables
/// The goal with this enum is to provide a way to access typed configuration from Environement
/// variables.
///
/// This will allow the type to be validated before it is used by the program.
///
/// ## Steps to add new Environment Variables:
/// 1. Add the key name to this enum.
/// 1. Add the new variant in the `as_str` impl
///   (use the name of the env var you would like to provide).
/// 1. Implement the 'From' trait. You should implement this for the value
///   that you would like the Env Var to be read as.
///
/// ### Valid Examples
/// This is what using an env variable for a boolean would look like.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// env::set_var(ConfigEnvKey::DevMode.as_str(), "true");
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
///
/// assert_eq!(is_dev_mode, true);
/// ```
///
/// And if no value is provided you can choose to add a default value.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // In this case the default for `ConfigEnvKey` is `false`.
/// env::remove_var(ConfigEnvKey::DevMode.as_str());
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
///
/// assert_eq!(is_dev_mode, false);
/// ```
/// ### Panic Examples
/// If you try to read an invalid value into your program, it *SHOULD* panic at config time.
/// ```should_panic
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // In this case the default for `ConfigEnvKey` is `false`.
/// env::set_var(ConfigEnvKey::DevMode.as_str(), "123not_bool");
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
/// ```
pub enum ConfigEnvKey {
    /// Url used to configure otlp service.
    OtelCollectorUrl,
    /// Port that the service will bind to.
    ServicePort,
    /// Ip that the service will bind to.
    ServiceIp,
    /// Determines if the app should be configured for development, or production.
    DevMode,
}

impl ConfigEnvKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigEnvKey::ServicePort => "SERVICE_PORT",
            ConfigEnvKey::ServiceIp => "SERVICE_IP",
            ConfigEnvKey::OtelCollectorUrl => "OTEL_COLLECTOR_URL",
            ConfigEnvKey::DevMode => "DEV_MODE",
        }
    }
}

const DEFAULT_PANIC_MSG: &str =
    "Check the impl block for the type you are trying to use and make sure the key is implemented.";

impl From<ConfigEnvKey> for u16 {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::ServicePort => {
                match env::var(ConfigEnvKey::ServicePort.as_str()) {
                    Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
                        panic!(
                            "{} should be a valid u16! To use default port unset {} environment variable.",
                            ConfigEnvKey::ServicePort.as_str(),
                            ConfigEnvKey::ServicePort.as_str(),
                        )
                    }),
                    Err(_) => Config::DEFAULT_SERVICE_PORT,
                }
            },
            _ => panic!("this key cannot be turned into a u16. {DEFAULT_PANIC_MSG}")
        }
    }
}

impl From<ConfigEnvKey> for Ipv4Addr {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::ServiceIp => {
                match env::var(ConfigEnvKey::ServiceIp.as_str()) {
                    Ok(aip) => {
                        aip.parse::<Ipv4Addr>().unwrap_or_else(|_| panic!("{} should be a valid Ipv4Addr! To use default ip unset {} environment variable.",
                                ConfigEnvKey::ServiceIp.as_str(),
                                ConfigEnvKey::ServiceIp.as_str()))
                    }
                    Err(_) => Config::DEFAULT_SERVICE_IP,
                }
            },
            _ => panic!("this key cannot be converted to Ipv4Addr. {DEFAULT_PANIC_MSG}")
        }
    }
}

/// This is what using an env variable for a boolean would look like.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // I am not using the literal here to avoid breaking tests if the name changes.
/// env::set_var(ConfigEnvKey::DevMode.as_str(), "true");
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
///
/// assert_eq!(is_dev_mode, true);
/// ```
impl From<ConfigEnvKey> for bool {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::DevMode => match env::var(ConfigEnvKey::DevMode.as_str()) {
                Ok(is_dev) => is_dev
                    .parse::<bool>()
                    .unwrap_or_else(|_| panic!("{} should be a valid bool!", is_dev)),
                Err(_) => Config::DEFAULT_DEV_MODE,
            },
            _ => panic!("this key cannot be converted to bool. {DEFAULT_PANIC_MSG}"),
        }
    }
}

/// This is what using an env variable for a String would look like.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // I am not using the literal here to avoid breaking tests if the name changes.
/// env::set_var(ConfigEnvKey::OtelCollectorUrl.as_str(), "tcp://localhost:4317");
///
/// let otel_col_url = String::from(ConfigEnvKey::OtelCollectorUrl);
///
/// assert_eq!(otel_col_url, "tcp://localhost:4317");
/// ```
impl From<ConfigEnvKey> for String {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::OtelCollectorUrl => {
                match env::var(ConfigEnvKey::OtelCollectorUrl.as_str()) {
                    Ok(otel_url) => otel_url,
                    Err(_) => Config::DEFAULT_OTEL_URL.to_string(),
                }
            }
            _ => panic!("this key cannot be converted to String. {DEFAULT_PANIC_MSG}"),
        }
    }
}
