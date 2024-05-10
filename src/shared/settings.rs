use config::{Config, ConfigError, Environment, File};
use getset::Getters;
use log::error;
use std::env::var;

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct JwtSettings {
    secret: String,
}

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct EntraSettings {
    tenant_id: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct DbSettings {
    connection_string: String,
    username: String,
    password: String,
    namespace: String,
    database: String,
}

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Settings {
    jwt: JwtSettings,
    entra: EntraSettings,
    db: DbSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::with_name("appsettings.json"))
            .add_source(Environment::with_prefix("APP").separator("__"));

        let cfg = builder.build()?;
        let settings = Settings::from_config(cfg)?;
        Ok(settings)
    }

    fn get_env_var(name: &str) -> Result<String, ConfigError> {
        var(name).map_err(|_| {
            error!("{} must be set", name);
            ConfigError::Message(format!("{} must be set", name))
        })
    }

    fn from_config(cfg: Config) -> Result<Self, ConfigError> {
        let jwt_secret = Self::get_env_var("JWT_SECRET")?;

        let jwt = JwtSettings { secret: jwt_secret };

        let tenant_id = cfg.get::<String>("entra.tenant_id")?;
        let client_id = cfg.get::<String>("entra.client_id")?;
        let client_secret = Self::get_env_var("ENTRA_CLIENT_SECRET")?;
        let redirect_uri = cfg.get::<String>("entra.redirect_uri")?;

        let entra = EntraSettings {
            tenant_id,
            client_id,
            client_secret,
            redirect_uri,
        };

        let connection_string = cfg.get::<String>("db.connection_string")?;
        let username = cfg.get::<String>("db.username")?;
        let password = Self::get_env_var("DB_PASSWORD")?;
        let namespace = cfg.get::<String>("db.namespace")?;
        let database = cfg.get::<String>("db.database")?;

        let db = DbSettings {
            connection_string,
            username,
            password,
            namespace,
            database,
        };

        Ok(Settings { jwt, entra, db })
    }
}
