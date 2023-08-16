use std::{env, net::Ipv4Addr};

use crate::config::Config;

pub enum ConfigEnvKey {
    OtelCollectorUrl,
    ServicePort,
    ServiceIp,
    DevMode,
}

impl ConfigEnvKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigEnvKey::ServicePort => "PORT",
            ConfigEnvKey::ServiceIp => "ACTIX_IP",
            ConfigEnvKey::OtelCollectorUrl => "OTEL_COLLECTOR_URL",
            ConfigEnvKey::DevMode => "DEV_MODE",
        }
    }
}

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
            _ => panic!("this key cannot be turned into a u16.")
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
            _ => panic!("this key cannot be converted to Ipv4Addr.")
        }
    }
}

impl From<ConfigEnvKey> for bool {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::DevMode => match env::var(ConfigEnvKey::DevMode.as_str()) {
                Ok(is_dev) => is_dev
                    .parse::<bool>()
                    .unwrap_or_else(|_| panic!("{} should be a valid bool!", is_dev)),
                Err(_) => Config::DEFAULT_DEV_MODE,
            },
            _ => panic!("this key cannot be converted to String"),
        }
    }
}

impl From<ConfigEnvKey> for String {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::OtelCollectorUrl => {
                match env::var(ConfigEnvKey::OtelCollectorUrl.as_str()) {
                    Ok(otel_url) => otel_url,
                    Err(_) => Config::DEFAULT_OTEL_URL.to_string(),
                }
            }
            _ => panic!("this key cannot be converted to String"),
        }
    }
}
