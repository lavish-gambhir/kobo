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

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    println!("{:?}", std::env::current_dir());
    let mut settings = config::Config::builder()
        .set_default("default", "1")
        .unwrap()
        .add_source(config::File::with_name("configuration"))
        .build()?;
    settings.try_deserialize()
}
