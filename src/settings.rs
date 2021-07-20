use once_cell::sync::Lazy;

#[derive(Debug, Default)]
pub struct AppSettings {
    pub db_uri: String,
    pub server_port: String,
}

pub static APP_SETTINGS: Lazy<AppSettings> = Lazy::new(|| {
    let mut settings = AppSettings::default();

    settings.db_uri = std::env::var("APP_DB_URI")
        .unwrap_or("mongodb://root:example@localhost:27017/".to_string());

    settings.server_port = std::env::var("APP_SERVER_PORT").unwrap_or("8080".to_string());

    settings
});
