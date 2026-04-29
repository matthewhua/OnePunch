use anyhow::Result;

pub mod building;
pub mod skin;
pub mod activity;

/// 玩家功能系统 trait
/// 每个系统管理一块玩家数据（对应 p_data 表的一行，按 keyId 区分）
pub trait PlayerSystem: Send + Sync {
    /// 从二进制数据加载（兼容 Java 版存储的 Blob）
    fn load_from_bin(&mut self, data: &[u8]) -> Result<()>;

    /// 序列化为二进制数据保存
    fn save_to_bin(&self) -> Result<Vec<u8>>;

    /// 是否有未存盘的变更
    fn is_dirty(&self) -> bool {
        false
    }

    /// 标记数据已变更
    fn mark_dirty(&mut self) {}

    /// 清除 dirty 标记（存盘后调用）
    fn clear_dirty(&mut self) {}

    /// 对应 p_data 表的 keyId（与 Java 版 FunctionTypeDefine 一致）
    fn key_id(&self) -> i32;

    /// 对应 p_data 表的列名（用于日志和调试）
    fn column_name(&self) -> &'static str;

    /// 每秒 tick（驱动倒计时、buff 过期等）
    fn tick(&mut self) {}

    /// 玩家登录时调用
    fn on_login(&mut self) {}

    /// 玩家下线时调用
    fn on_logout(&mut self) {}

    /// 每日重置（跨天时调用）
    fn on_daily_reset(&mut self) {}
}

/// 兼容 Java 版 FunctionClientBase 体系的特征
/// 每个需要推送到客户端的功能模块都需要实现此特征
/// 使用 shared::msg::ToFunctionClientBaseBytes 的统一实现
pub use shared::msg::ToFunctionClientBaseBytes as ToFunctionClientBase;

