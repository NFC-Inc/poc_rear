use dotenv::dotenv;
use mongodb::options::ClientOptions;
use tracing::metadata::LevelFilter;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::Resource;
use opentelemetry::{global, sdk as sdktrace, KeyValue};

use tracing_subscriber::{prelude::*, EnvFilter};

use opentelemetry_otlp::WithExportConfig;

use crate::config_env::ConfigEnvKey;
use crate::user_models::User;
use crate::wotd_models::DisplayWotdDto;

pub struct Config {
    using_dotenv_path: Option<PathBuf>,
    service_ip: Ipv4Addr,
    service_port: u16,
    otel_url: String,
}

impl Config {
    // Used for consistent span keys.
    // pub const LEMONS_KEY: Key = Key::from_static_str("lemons");
    // pub const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

    pub const APP_NAME: &str = "poc_rear";
    pub const DEFAULT_SERVICE_PORT: u16 = 8080;
    pub const DEFAULT_SERVICE_IP: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    pub const DEFAULT_OTEL_URL: &str = "https://localhost:4317";
    pub const DEFAULT_LOG_FILTER: &str = "INFO";
    pub const DEFAULT_TRACE_FILTER: &str =
        "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug";

    pub const MONGO_DB_NAME: &str = Config::APP_NAME;
    pub const MONGO_COLL_NAME_WOTD: &str = "wotd";
    pub const MONGO_COLL_NAME_USER: &str = "user";
    pub const DEVELOPMENT: bool = true;

    pub fn new() -> Config {
        Config {
            using_dotenv_path: dotenv().ok(),
            service_ip: Ipv4Addr::from(ConfigEnvKey::ServiceIp),
            service_port: u16::from(ConfigEnvKey::ServicePort),
            otel_url: String::from(ConfigEnvKey::OtelCollectorUrl),
        }
    }

    pub fn log_config_values(&self, level: log::Level) {
        if let Some(path) = self.dotenv_path() {
            log::log!(level, "Using .env file from: [{:?}]", path);
        }
        log::log!(
            level,
            "Listening on        : [{}:{}]",
            self.service_ip(),
            self.service_port()
        );
        log::log!(
            level,
            "Sending ({}) traces to   : [{}]",
            Config::DEFAULT_TRACE_FILTER,
            self.otel_url()
        );
    }

    pub fn service_ip(&self) -> Ipv4Addr {
        self.service_ip
    }

    pub fn service_port(&self) -> u16 {
        self.service_port
    }

    pub fn otel_url(&self) -> String {
        self.otel_url.to_string()
    }

    pub fn dotenv_path(&self) -> Option<PathBuf> {
        self.using_dotenv_path.to_owned()
    }

    pub fn init_otel() {
        global::set_text_map_propagator(TraceContextPropagator::new());

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(String::from(ConfigEnvKey::OtelCollectorUrl)),
            )
            .with_trace_config(sdktrace::trace::config().with_resource(Resource::new(vec![
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME.to_string(),
                    Config::APP_NAME,
                ),
            ])))
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or(EnvFilter::new(Config::DEFAULT_LOG_FILTER)),
                ),
            )
            .with(
                tracing_opentelemetry::layer()
                    .with_tracer(tracer)
                    .with_filter(EnvFilter::from_default_env()),
            )
            .try_init()
            .unwrap();
    }

    pub async fn init_mongo() -> mongodb::Client {
        let uri =
            std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
        let client = mongodb::Client::with_uri_str(uri)
            .await
            .expect("failed to connect");
        let options = mongodb::options::IndexOptions::builder()
            .unique(true)
            .build();

        let wotd_model = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "word": 1 })
            .options(options.clone())
            .build();

        client
            .database(Config::MONGO_DB_NAME)
            .collection::<DisplayWotdDto>(Config::MONGO_COLL_NAME_WOTD)
            .create_index(wotd_model, None)
            .await
            .expect("creating an index should succeed");

        let user_model = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "username": 1 })
            .options(options.clone())
            .build();

        client
            .database(Config::MONGO_DB_NAME)
            .collection::<User>(Config::MONGO_COLL_NAME_USER)
            .create_index(user_model, None)
            .await
            .expect("creating database index for users should work");

        client
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_u16_from_env_key_actix_port_default() {
        // Arrange
        env::remove_var(ConfigEnvKey::ServicePort.as_str());

        // Act / Assert
        assert_eq!(
            Config::DEFAULT_SERVICE_PORT,
            u16::from(ConfigEnvKey::ServicePort)
        );
    }

    #[test]
    fn test_u16_from_env_key_actix_port_non_default() {
        // Arrange
        env::set_var(ConfigEnvKey::ServicePort.as_str(), "1337");

        // Act / Assert
        assert_eq!(1337, u16::from(ConfigEnvKey::ServicePort));

        // Cleanup
        env::remove_var(ConfigEnvKey::ServicePort.as_str())
    }

    #[test]
    fn test_ipv4_from_env_key_actix_ip_default() {
        // Arrange
        env::remove_var(ConfigEnvKey::ServiceIp.as_str());

        // Act / Assert
        assert_eq!(
            Config::DEFAULT_SERVICE_IP,
            Ipv4Addr::from(ConfigEnvKey::ServiceIp)
        );
    }

    #[test]
    fn test_ipv4_from_env_key_actix_ip_non_default() {
        // Arrange
        env::set_var(ConfigEnvKey::ServiceIp.as_str(), "127.0.0.1");

        // Act / Assert
        assert_eq!(
            Ipv4Addr::new(127, 0, 0, 1),
            Ipv4Addr::from(ConfigEnvKey::ServiceIp)
        );

        // Cleanup
        env::remove_var(ConfigEnvKey::ServiceIp.as_str())
    }

    #[test]
    fn test_string_from_env_key_otel_url_default() {
        // Arrange
        env::remove_var(ConfigEnvKey::OtelCollectorUrl.as_str());

        // Act / Assert
        assert_eq!(
            Config::DEFAULT_OTEL_URL,
            String::from(ConfigEnvKey::OtelCollectorUrl)
        );
    }

    #[test]
    fn test_string_from_env_key_otel_url_non_default() {
        // Arrange
        env::set_var(ConfigEnvKey::OtelCollectorUrl.as_str(), "127.0.0.1:8080");

        // Act / Assert
        assert_eq!(
            "127.0.0.1:8080",
            String::from(ConfigEnvKey::OtelCollectorUrl)
        );

        // Cleanup
        env::remove_var(ConfigEnvKey::OtelCollectorUrl.as_str())
    }

    #[test]
    fn test_config_new() {
        // Arrange / Act
        let new_config = Config::new();

        // Assert
        assert_eq!(Config::DEFAULT_SERVICE_IP, new_config.service_ip());
        assert_eq!(Config::DEFAULT_SERVICE_PORT, new_config.service_port());
        assert_eq!(Config::DEFAULT_OTEL_URL, new_config.otel_url());
    }
}
