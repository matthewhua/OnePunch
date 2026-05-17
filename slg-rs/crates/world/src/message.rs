use anyhow::Result;
use bytes::Bytes;
use proto::slg::{BaseEntity, BaseTroop};
use tokio::sync::oneshot;

use crate::collect::CollectState;

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
        troop_data: BaseTroop,
        formation_id: Option<i32>,
        collect_state: Option<CollectState>,
    },
    /// 部队状态更新（召回、加速等）
    UpdateTroop { troop_data: BaseTroop },
    /// 从本 Sector 视图移除部队
    RemoveTroop { troop_key: i32 },
    /// 同步实体到本 Sector 视图
    UpsertEntity { entity_data: BaseEntity },
    /// 从本 Sector 视图移除实体
    RemoveEntity { pos: i32 },
    /// 定时 Tick (100ms)
    Tick,
    /// 配置热加载
    ConfigReload,
}
