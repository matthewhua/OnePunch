use anyhow::Result;
use prost::Message;

pub mod building;
pub mod skin;
pub mod activity;

/// 玩家子系统组件基类特征
/// 所有系统（建筑、背包、皮肤等）都需要实现此特征以支持数据落盘
pub trait PlayerSystem: Send + Sync {
    /// 从二进制数据加载（兼容 Java 版存储的 Blob）
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()>;
    
    /// 序列化为二进制数据保存
    fn save_to_bin(&self) -> Result<Vec<u8>>;
}
