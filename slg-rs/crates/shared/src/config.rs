use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_addr: String,
    // 其他配置项...
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // 加载基础配置
            .add_source(File::with_name("config/base").required(false))
            // 加载环境变量覆盖
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        s.try_deserialize()
    }
}
