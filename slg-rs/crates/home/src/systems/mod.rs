use anyhow::Result;
use prost::Message;

pub mod building;
pub mod skin;
pub mod activity;

pub trait PlayerSystem: Send + Sync {
    /// 从二进制数据加载（兼容 Java 版存储的 Blob）
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()>;
    
    /// 序列化为二进制数据保存
    fn save_to_bin(&self) -> Result<Vec<u8>>;
}

/// 兼容 Java 版 FunctionClientBase 体系的特征
/// 每个需要推送到客户端的功能模块（Activity, Hero, Building 等）都需要实现此特征
pub trait ToFunctionClientBase {
    fn to_function_base_bytes(&self) -> Result<Vec<u8>>;
}

