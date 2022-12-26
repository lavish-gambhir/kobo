//! src/configuration.rs

use config::builder::DefaultState;
use config::FileFormat;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::builder()
        .set_default("default", "1")
        .unwrap()
        .add_source(config::File::with_name("configuration"))
        .build()?;
    settings.try_deserialize()
}
