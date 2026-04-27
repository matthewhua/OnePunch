use bytes::Bytes;
use tokio::sync::oneshot;
use anyhow::Result;
use proto::slg::BaseTroop; // 假设 proto crate 中有这个

pub enum SectorMessage {
    /// 玩家命令（派兵、召回等）
    PlayerCommand {
        role_id: i64,
        cmd: u32,
        payload: Bytes,
        reply: oneshot::Sender<Result<Bytes>>,
    },
    /// 跨区部队转移
    TransferTroop {
        troop_data: BaseTroop, // 这里可以先用 BaseTroop，以后根据需求扩展
    },
    /// 定时 Tick (100ms)
    Tick,
    /// 配置热加载
    ConfigReload,
}
