use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_addr: String,
    
    // 集群服务地址
    pub auth_service_addr: String,
    pub home_service_addr: String,
    pub world_service_addr: String,

    // Etcd 配置
    pub etcd_endpoints: Vec<String>,
    // 当前服务的唯一 ID
    pub server_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "mysql://root:password@localhost:3306/slg".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            server_addr: "0.0.0.0:8080".to_string(),
            auth_service_addr: "http://127.0.0.1:50051".to_string(),
            home_service_addr: "http://127.0.0.1:50052".to_string(),
            world_service_addr: "http://127.0.0.1:50053".to_string(),
            etcd_endpoints: vec!["http://127.0.0.1:2379".to_string()],
            server_id: "default-1".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ::config::ConfigError> {
        let s = ::config::Config::builder()
            // 加载基础配置
            .add_source(::config::File::with_name("config/base").required(false))
            // 加载环境变量覆盖
            .add_source(::config::Environment::with_prefix("APP"))
            .build()?;

        s.try_deserialize()
    }
}
