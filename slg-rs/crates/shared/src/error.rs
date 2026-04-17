use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Protobuf decode error: {0}")]
    DecodeError(#[from] prost::DecodeError),
    
    #[error("Unknown command: {0}")]
    UnknownCommand(u32),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, GameError>;
