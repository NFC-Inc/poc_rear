use std::{env, net::Ipv4Addr};

use crate::config::Config;

pub enum ConfigEnvKey {
    OtelCollectorUrl,
    ActixPort,
    ActixIp,
}

impl ConfigEnvKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigEnvKey::ActixPort => "PORT",
            ConfigEnvKey::ActixIp => "ACTIX_IP",
            ConfigEnvKey::OtelCollectorUrl => "OTEL_COLLECTOR_URL",
        }
    }
}

impl From<ConfigEnvKey> for u16 {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::ActixPort => {
                match env::var(ConfigEnvKey::ActixPort.as_str()) {
                    Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
                        panic!(
                            "{} should be a valid u16! To use default port unset {} environment variable.",
                            ConfigEnvKey::ActixPort.as_str(),
                            ConfigEnvKey::ActixPort.as_str(),
                        )
                    }),
                    Err(_) => Config::DEFAULT_ACTIX_PORT,
                }
            },
            _ => panic!("this key cannot be turned into a u16.")
        }
    }
}

impl From<ConfigEnvKey> for Ipv4Addr {
    fn from(env_key: ConfigEnvKey) -> Self {
        match env_key {
            ConfigEnvKey::ActixIp => {
                match env::var(ConfigEnvKey::ActixIp.as_str()) {
                    Ok(aip) => {
                        aip.parse::<Ipv4Addr>().unwrap_or_else(|_| panic!("{} should be a valid Ipv4Addr! To use default ip unset {} environment variable.",
                                ConfigEnvKey::ActixIp.as_str(),
                                ConfigEnvKey::ActixIp.as_str()))
                    }
                    Err(_) => Config::DEFAULT_ACTIX_IP,
                }
            },
            _ => panic!("this key cannot be converted to Ipv4Addr.")
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
