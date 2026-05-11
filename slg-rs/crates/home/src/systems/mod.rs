use anyhow::Result;
use std::sync::Arc;
use shared::static_config::StaticConfig;

pub mod activity;
pub mod hero;
pub mod backpack;
pub mod building;
pub mod tech;
pub mod equip;
pub mod mission;
pub mod skin;

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
    fn is_dirty(&self) -> bool { false }

    /// 标记数据已变更
    fn mark_dirty(&mut self) {}

    /// 清除 dirty 标记（存盘后调用）
    fn clear_dirty(&mut self) {}

    /// 对应 p_data 表的列名（如 "activity_func"）
    fn column_name(&self) -> &'static str;

    /// 每秒 tick（驱动倒计时、buff 过期等）
    fn tick(&mut self) {}

    /// 玩家登录时调用
    fn on_login(&mut self) {}

    /// 玩家下线时调用
    fn on_logout(&mut self) {}

    /// 每日重置（跨天时调用）
    fn on_daily_reset(&mut self) {}

    /// 处理业务命令，返回序列化后的响应 payload。
    ///
    /// 默认实现返回 "未知命令" 错误，各系统按需覆盖。
    fn handle_command(
        &mut self,
        cmd: u32,
        _payload: &[u8],
        _config: &Arc<StaticConfig>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!(
            "System '{}' does not handle cmd {}",
            self.column_name(),
            cmd
        ))
    }

    /// 处理业务命令，同时返回需要分发的游戏事件列表。
    ///
    /// 默认实现调用 `handle_command` 并返回空事件列表。
    /// 需要触发事件的系统应覆盖此方法。
    fn handle_command_with_events(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<(Vec<u8>, Vec<shared::event::GameEvent>)> {
        let resp = self.handle_command(cmd, payload, config)?;
        Ok((resp, vec![]))
    }
}

/// 兼容 Java 版 FunctionClientBase 体系的特征
pub use shared::msg::ToFunctionClientBaseBytes as ToFunctionClientBase;
