use dotenv::dotenv;
use tracing::metadata::LevelFilter;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::Resource;
use opentelemetry::{global, sdk as sdktrace, KeyValue};

use tracing_subscriber::{prelude::*, EnvFilter};

use opentelemetry_otlp::WithExportConfig;

use crate::config_env::ConfigEnvKey;

pub struct Config {
    using_dotenv_path: Option<PathBuf>,
    actix_ip: Ipv4Addr,
    actix_port: u16,
    otel_url: String,
}

impl Config {
    // Used for consistent span keys.
    // pub const LEMONS_KEY: Key = Key::from_static_str("lemons");
    // pub const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

    pub const APP_NAME: &str = "poc_rear";
    pub const DEFAULT_ACTIX_PORT: u16 = 8080;
    pub const DEFAULT_ACTIX_IP: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    pub const DEFAULT_OTEL_URL: &str = "https://localhost:4317";
    pub const DEFAULT_LOG_FILTER: &str = "INFO";
    pub const DEFAULT_TRACE_FILTER: &str = "INFO";

    pub const MONGO_DB_NAME: &str = Config::APP_NAME;
    pub const MONGO_COLL_NAME: &str = "users";

    pub fn new() -> Config {
        Config {
            using_dotenv_path: dotenv().ok(),
            actix_ip: Ipv4Addr::from(ConfigEnvKey::ActixIp),
            actix_port: u16::from(ConfigEnvKey::ActixPort),
            otel_url: String::from(ConfigEnvKey::OtelCollectorUrl),
        }
    }

    pub fn print_values(&self, level: log::Level) {
        if let Some(path) = self.dotenv_path() {
            log::log!(level, "Using .env file from: [{:?}]", path);
        }
        log::log!(
            level,
            "Listening on        : [{}:{}]",
            self.actix_ip(),
            self.actix_port()
        );
        log::log!(level, "Sending traces to   : [{}]", self.otel_url());
    }

    pub fn actix_ip(&self) -> Ipv4Addr {
        self.actix_ip
    }

    pub fn actix_port(&self) -> u16 {
        self.actix_port
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

    pub async fn init_mongo<T>() -> mongodb::Client {
        let uri =
            std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
        let client = mongodb::Client::with_uri_str(uri)
            .await
            .expect("failed to connect");
        let options = mongodb::options::IndexOptions::builder()
            .unique(true)
            .build();
        let model = mongodb::IndexModel::builder()
            .keys(mongodb::bson::doc! { "username": 1 })
            .options(options)
            .build();

        client
            .database(Config::MONGO_DB_NAME)
            .collection::<T>(Config::MONGO_COLL_NAME)
            .create_index(model, None)
            .await
            .expect("creating an index should succeed");

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
        env::remove_var(ConfigEnvKey::ActixPort.as_str());

        // Act / Assert
        assert_eq!(
            Config::DEFAULT_ACTIX_PORT,
            u16::from(ConfigEnvKey::ActixPort)
        );
    }

    #[test]
    fn test_u16_from_env_key_actix_port_non_default() {
        // Arrange
        env::set_var(ConfigEnvKey::ActixPort.as_str(), "1337");

        // Act / Assert
        assert_eq!(1337, u16::from(ConfigEnvKey::ActixPort));

        // Cleanup
        env::remove_var(ConfigEnvKey::ActixPort.as_str())
    }

    #[test]
    fn test_ipv4_from_env_key_actix_ip_default() {
        // Arrange
        env::remove_var(ConfigEnvKey::ActixIp.as_str());

        // Act / Assert
        assert_eq!(
            Config::DEFAULT_ACTIX_IP,
            Ipv4Addr::from(ConfigEnvKey::ActixIp)
        );
    }

    #[test]
    fn test_ipv4_from_env_key_actix_ip_non_default() {
        // Arrange
        env::set_var(ConfigEnvKey::ActixIp.as_str(), "127.0.0.1");

        // Act / Assert
        assert_eq!(
            Ipv4Addr::new(127, 0, 0, 1),
            Ipv4Addr::from(ConfigEnvKey::ActixIp)
        );

        // Cleanup
        env::remove_var(ConfigEnvKey::ActixIp.as_str())
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
        assert_eq!(Config::DEFAULT_ACTIX_IP, new_config.actix_ip());
        assert_eq!(Config::DEFAULT_ACTIX_PORT, new_config.actix_port());
        assert_eq!(Config::DEFAULT_OTEL_URL, new_config.otel_url());
    }
}
