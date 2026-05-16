use proto::slg::world_service_server::WorldService;
use proto::slg::{
    BaseMap, BasePlayerMapData, BaseTroop, DispatchPigeonTroopRq, DispatchRq, DispatchRs,
    DispatchScoutTroopRq, DispatchTroopRq, EnterWorldMapRq, GetAreaDetailsRq, GetAreaDetailsRs,
    GetBlockDetailsRq, GetBlockDetailsRs, GetEntityInfoRq, GetEntityInfoRs, GetFightDetailsRs,
    GetFightDetailsRq, GetMapDetailsRq, GetMapDetailsRs, GetPlayerTroopRs, GetTroopDetailsRs,
    GetTroopDetailsRq, GetTroopInfoRq, GetTroopInfoRs, JoinMapRequest, JoinMapResponse,
    LeaveWorldMapRs, MovePositionRq, MovePositionRs, RpcMsg,
};
use shared::msg::GameMessage;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct WorldServiceImpl {
    grid: Arc<crate::map::grid::MapGrid>,
    marching_mgr: Arc<crate::march::MarchingManager>,
    next_troop_key: AtomicI32,
}

impl WorldServiceImpl {
    pub fn new(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
    ) -> Self {
        Self {
            grid,
            marching_mgr,
            next_troop_key: AtomicI32::new(1),
        }
    }

    fn response<T: prost::Message>(&self, cmd: i32, msg: &T) -> Result<DispatchRs, Status> {
        let payload = GameMessage::build_response(cmd, msg)
            .map_err(|e| Status::internal(format!("encode world response failed: {}", e)))?;
        Ok(DispatchRs { code: 0, payload })
    }

    fn decode_payload<T: prost::Message + Default>(&self, cmd: i32, payload: Vec<u8>) -> Result<T, Status> {
        let msg = GameMessage::decode(payload)
            .map_err(|e| Status::invalid_argument(format!("decode world request failed: {}", e)))?;
        if msg.base.cmd != cmd {
            return Err(Status::invalid_argument(format!(
                "world request cmd mismatch: frame={}, base={}",
                cmd, msg.base.cmd
            )));
        }
        msg.get_payload()
            .map_err(|e| Status::invalid_argument(format!("decode world payload failed: {}", e)))
    }

    fn dispatch_troop(
        &self,
        role_id: i64,
        target_pos: i32,
        troop_type: i32,
    ) -> Result<BaseTroop, Status> {
        if !crate::map::grid::is_valid_pos(target_pos) {
            return Err(Status::invalid_argument(format!(
                "invalid world target position: {}",
                target_pos
            )));
        }

        let key = self.next_troop_key.fetch_add(1, Ordering::Relaxed);
        let troop = BaseTroop {
            key,
            r#type: Some(troop_type),
            origin: Some(0),
            goal: Some(target_pos),
            camp: Some(0),
            ..Default::default()
        };

        self.marching_mgr
            .start_march(troop, 10.0)
            .map_err(|e| Status::failed_precondition(format!("start march failed: {}", e)))
    }

    fn troop_type_for_entity(entity_type: Option<i32>) -> i32 {
        match entity_type.unwrap_or_default() {
            1 => crate::march::MARCH_TYPE_ATK_PLAYER,
            3 => crate::march::MARCH_TYPE_MINE_COLLECT,
            4 => crate::march::MARCH_TYPE_ATK_CITY,
            _ => crate::march::MARCH_TYPE_ATK_BANDIT,
        }
    }

    fn all_marching_troops(&self) -> Vec<BaseTroop> {
        let mut troops: Vec<BaseTroop> = self
            .marching_mgr
            .troops
            .iter()
            .map(|entry| entry.value().base.clone())
            .collect();
        troops.sort_by_key(|troop| troop.key);
        troops
    }
}

#[tonic::async_trait]
impl WorldService for WorldServiceImpl {
    async fn call(&self, _request: Request<RpcMsg>) -> Result<Response<RpcMsg>, Status> {
        // 通用 RPC 入口实现
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn dispatch(&self, request: Request<DispatchRq>) -> Result<Response<DispatchRs>, Status> {
        let req = request.into_inner();
        let rs = match req.cmd {
            50001 => {
                let rq: GetMapDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                let map_id = rq.map_id.unwrap_or(1);
                self.response(50002, &GetMapDetailsRs {
                    map: vec![BaseMap {
                        map: map_id,
                        ..Default::default()
                    }],
                })
            }
            50003 => {
                let rq: EnterWorldMapRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50004, &proto::slg::EnterWorldMapRs {
                    player_data: Some(BasePlayerMapData {
                        map: rq.map_id,
                        role_id: req.role_id,
                        pos: 0,
                        ..Default::default()
                    }),
                })
            }
            50005 => self.response(50006, &LeaveWorldMapRs::default()),
            50007 => {
                let _rq: GetAreaDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50008, &GetAreaDetailsRs {
                    entity: self.grid.all_entities(),
                })
            }
            50009 => {
                let rq: GetBlockDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50010, &GetBlockDetailsRs {
                    entity: self.grid.all_entities(),
                    block: rq.block,
                })
            }
            50011 => {
                let rq: GetEntityInfoRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50012, &GetEntityInfoRs {
                    entity: self.grid.get_entity(rq.pos),
                })
            }
            50013 => {
                let rq: MovePositionRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50014, &MovePositionRs {
                    player_data: Some(BasePlayerMapData {
                        map: rq.map,
                        role_id: req.role_id,
                        pos: rq.pos.unwrap_or(0),
                        ..Default::default()
                    }),
                })
            }
            50015 => {
                let _rq: GetTroopDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50016, &GetTroopDetailsRs {
                    march_troop: self.all_marching_troops(),
                })
            }
            50017 => {
                let _rq: GetFightDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50018, &GetFightDetailsRs::default())
            }
            50019 => {
                let rq: DispatchTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self.dispatch_troop(
                    req.role_id,
                    rq.pos,
                    Self::troop_type_for_entity(rq.r#type),
                )?;
                self.response(50020, &proto::slg::DispatchTroopRs::default())
            }
            50031 => self.response(50032, &GetPlayerTroopRs {
                troop: self.all_marching_troops(),
            }),
            50035 => {
                let rq: GetTroopInfoRq = self.decode_payload(req.cmd, req.payload)?;
                let troop = self
                    .marching_mgr
                    .troops
                    .get(&rq.troop_key)
                    .map(|entry| entry.value().base.clone());
                self.response(50036, &GetTroopInfoRs { troop })
            }
            50037 => {
                let rq: DispatchPigeonTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self.dispatch_troop(
                    req.role_id,
                    rq.pos,
                    crate::march::MARCH_TYPE_INTEL_TASK,
                )?;
                self.response(50038, &proto::slg::DispatchPigeonTroopRs::default())
            }
            50039 => {
                let rq: DispatchScoutTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self.dispatch_troop(
                    req.role_id,
                    rq.pos,
                    crate::march::MARCH_TYPE_SCOUT,
                )?;
                self.response(50040, &proto::slg::DispatchScoutTroopRs::default())
            }
            _ => {
                tracing::warn!(
                    role_id = req.role_id,
                    cmd = req.cmd,
                    payload_len = req.payload.len(),
                    "World Dispatch received unsupported command"
                );
                Ok(DispatchRs {
                    code: 501,
                    payload: vec![],
                })
            }
        }?;

        Ok(Response::new(rs))
    }

    async fn join_map(
        &self,
        request: Request<JoinMapRequest>,
    ) -> Result<Response<JoinMapResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Player {} joining map {}", req.role_id, req.map_id);

        // TODO: 设置玩家初始坐标并加入 AOI

        Ok(Response::new(JoinMapResponse {
            code: 0,
            msg: "Success".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use proto::slg::{DispatchTroopRs, WorldEntityTypeDefine};

    fn service() -> WorldServiceImpl {
        WorldServiceImpl::new(
            Arc::new(crate::map::grid::MapGrid::new()),
            Arc::new(crate::march::MarchingManager::new()),
        )
    }

    fn request<T: prost::Message>(cmd: i32, role_id: i64, body: &T) -> DispatchRq {
        DispatchRq {
            role_id,
            cmd,
            payload: GameMessage::build_response(cmd, body).unwrap(),
        }
    }

    #[tokio::test]
    async fn dispatch_troop_starts_march_and_returns_success() {
        let svc = service();
        let rq = DispatchTroopRq {
            map: 1,
            pos: crate::map::grid::xy_to_pos(10, 10),
            formation_id: 7,
            r#type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
        };

        let rs = svc.dispatch(Request::new(request(50019, 42, &rq))).await.unwrap().into_inner();

        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 50020);
        let _: DispatchTroopRs = msg.get_payload().unwrap();

        let troops = svc.all_marching_troops();
        assert_eq!(troops.len(), 1);
        assert_eq!(troops[0].goal, Some(rq.pos));
        assert_eq!(troops[0].r#type, Some(crate::march::MARCH_TYPE_MINE_COLLECT));
    }

    #[tokio::test]
    async fn get_player_troop_returns_started_marches() {
        let svc = service();
        svc.dispatch_troop(42, crate::map::grid::xy_to_pos(1, 1), crate::march::MARCH_TYPE_SCOUT)
            .unwrap();

        let rs = svc.dispatch(Request::new(request(50031, 42, &proto::slg::GetPlayerTroopRq::default())))
            .await
            .unwrap()
            .into_inner();
        let msg = GameMessage::decode(rs.payload).unwrap();
        let body: GetPlayerTroopRs = msg.get_payload().unwrap();

        assert_eq!(msg.base.cmd, 50032);
        assert_eq!(body.troop.len(), 1);
        assert_eq!(body.troop[0].r#type, Some(crate::march::MARCH_TYPE_SCOUT));
    }

    #[tokio::test]
    async fn dispatch_troop_rejects_invalid_target() {
        let svc = service();
        let rq = DispatchTroopRq {
            map: 1,
            pos: -1,
            formation_id: 7,
            r#type: Some(WorldEntityTypeDefine::EntityTypeBandit as i32),
        };

        let err = svc.dispatch(Request::new(request(50019, 42, &rq))).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
}
