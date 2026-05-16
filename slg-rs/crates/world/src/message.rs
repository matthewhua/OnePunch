use anyhow::Result;
use bytes::Bytes;
use proto::slg::BaseTroop;
use tokio::sync::oneshot;

pub enum SectorMessage {
    /// 玩家命令（派兵、召回等）
    PlayerCommand {
        role_id: i64,
        cmd: u32,
        payload: Bytes,
        reply: oneshot::Sender<Result<Bytes>>,
    },
    /// 跨区部队转移
    TransferTroop { troop_data: BaseTroop },
    /// 部队状态更新（召回、加速等）
    UpdateTroop { troop_data: BaseTroop },
    /// 从本 Sector 视图移除部队
    RemoveTroop { troop_key: i32 },
    /// 定时 Tick (100ms)
    Tick,
    /// 配置热加载
    ConfigReload,
}
