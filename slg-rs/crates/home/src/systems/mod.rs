use anyhow::Result;

pub mod building;
pub mod skin;
pub mod activity;

/// 玩家功能系统 trait
///
/// 每个系统管理一块玩家数据，对应 `p_data` 表的一个 blob 列。
/// 列名通过 `column_name()` 返回，与 Java 版 FunctionEntity 字段名完全一致。
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

    /// 对应 p_data 表的列名（如 "activity_func"）
    ///
    /// 与 `shared::persistence::col` 中的常量一致。
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
pub use shared::msg::ToFunctionClientBaseBytes as ToFunctionClientBase;
