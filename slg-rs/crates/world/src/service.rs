use proto::slg::world_service_server::WorldService;
use proto::slg::{
    BaseMap, BasePlayerMapData, BaseTroop, DispatchPigeonTroopRq, DispatchRq, DispatchRs,
    DispatchScoutTroopRq, DispatchTroopRq, EnterWorldMapRq, GetAreaDetailsRq, GetAreaDetailsRs,
    GetBlockDetailsRq, GetBlockDetailsRs, GetEntityInfoRq, GetEntityInfoRs, GetFightDetailsRq,
    GetFightDetailsRs, GetMapDetailsRq, GetMapDetailsRs, GetPlayerTroopRs, GetTroopDetailsRq,
    GetTroopDetailsRs, GetTroopInfoRq, GetTroopInfoRs, JoinMapRequest, JoinMapResponse,
    LeaveWorldMapRs, MovePositionRq, MovePositionRs, RpcMsg, SearchEntityRq, SearchEntityRs,
    TroopAccelerateCommandRq, TroopAccelerateCommandRs, TroopBackCommandRq, TroopBackCommandRs,
};
use shared::msg::GameMessage;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct WorldServiceImpl {
    grid: Arc<crate::map::grid::MapGrid>,
    marching_mgr: Arc<crate::march::MarchingManager>,
    runtime: Arc<crate::runtime::WorldRuntime>,
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
            runtime: Arc::new(crate::runtime::WorldRuntime::new()),
            next_troop_key: AtomicI32::new(1),
        }
    }

    fn response<T: prost::Message>(&self, cmd: i32, msg: &T) -> Result<DispatchRs, Status> {
        let payload = GameMessage::build_response(cmd, msg)
            .map_err(|e| Status::internal(format!("encode world response failed: {}", e)))?;
        Ok(DispatchRs { code: 0, payload })
    }

    fn decode_payload<T: prost::Message + Default>(
        &self,
        cmd: i32,
        payload: Vec<u8>,
    ) -> Result<T, Status> {
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

    async fn dispatch_troop(
        &self,
        _role_id: i64,
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

        let troop = self
            .marching_mgr
            .start_march(troop, 10.0)
            .map_err(|e| Status::failed_precondition(format!("start march failed: {}", e)))?;

        self.runtime
            .send_transfer_troop(troop.clone())
            .await
            .map_err(|e| Status::failed_precondition(format!("sector dispatch failed: {}", e)))?;

        Ok(troop)
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

    #[cfg(test)]
    fn sector_troop_count(&self, pos: i32) -> usize {
        self.runtime
            .sector_troop_count(crate::runtime::WorldRuntime::sector_id_for_pos(pos))
    }

    #[cfg(test)]
    fn sector_troop_keys(&self, pos: i32) -> Vec<i32> {
        self.runtime
            .sector_troop_keys(crate::runtime::WorldRuntime::sector_id_for_pos(pos))
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
                self.response(
                    50002,
                    &GetMapDetailsRs {
                        map: vec![BaseMap {
                            map: map_id,
                            ..Default::default()
                        }],
                    },
                )
            }
            50003 => {
                let rq: EnterWorldMapRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50004,
                    &proto::slg::EnterWorldMapRs {
                        player_data: Some(BasePlayerMapData {
                            map: rq.map_id,
                            role_id: req.role_id,
                            pos: 0,
                            ..Default::default()
                        }),
                    },
                )
            }
            50005 => self.response(50006, &LeaveWorldMapRs::default()),
            50007 => {
                let _rq: GetAreaDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50008,
                    &GetAreaDetailsRs {
                        entity: self.grid.all_entities(),
                    },
                )
            }
            50009 => {
                let rq: GetBlockDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50010,
                    &GetBlockDetailsRs {
                        entity: self.grid.all_entities(),
                        block: rq.block,
                    },
                )
            }
            50011 => {
                let rq: GetEntityInfoRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50012,
                    &GetEntityInfoRs {
                        entity: self.grid.get_entity(rq.pos),
                    },
                )
            }
            50013 => {
                let rq: MovePositionRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50014,
                    &MovePositionRs {
                        player_data: Some(BasePlayerMapData {
                            map: rq.map,
                            role_id: req.role_id,
                            pos: rq.pos.unwrap_or(0),
                            ..Default::default()
                        }),
                    },
                )
            }
            50015 => {
                let _rq: GetTroopDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50016,
                    &GetTroopDetailsRs {
                        march_troop: self.all_marching_troops(),
                    },
                )
            }
            50017 => {
                let _rq: GetFightDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(50018, &GetFightDetailsRs::default())
            }
            50019 => {
                let rq: DispatchTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self
                    .dispatch_troop(req.role_id, rq.pos, Self::troop_type_for_entity(rq.r#type))
                    .await?;
                self.response(50020, &proto::slg::DispatchTroopRs::default())
            }
            50027 => {
                let rq: TroopBackCommandRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self
                    .marching_mgr
                    .recall_troop(rq.key_id, rq.r#type)
                    .map_err(|e| {
                        Status::failed_precondition(format!("recall troop failed: {}", e))
                    })?;
                self.response(50028, &TroopBackCommandRs::default())
            }
            50029 => {
                let rq: TroopAccelerateCommandRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self
                    .marching_mgr
                    .accelerate_troop(rq.key_id, rq.r#type)
                    .map_err(|e| {
                        Status::failed_precondition(format!("accelerate troop failed: {}", e))
                    })?;
                self.response(50030, &TroopAccelerateCommandRs::default())
            }
            50031 => self.response(
                50032,
                &GetPlayerTroopRs {
                    troop: self.all_marching_troops(),
                },
            ),
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
                let _troop = self
                    .dispatch_troop(req.role_id, rq.pos, crate::march::MARCH_TYPE_INTEL_TASK)
                    .await?;
                self.response(50038, &proto::slg::DispatchPigeonTroopRs::default())
            }
            50039 => {
                let rq: DispatchScoutTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self
                    .dispatch_troop(req.role_id, rq.pos, crate::march::MARCH_TYPE_SCOUT)
                    .await?;
                self.response(50040, &proto::slg::DispatchScoutTroopRs::default())
            }
            5201 => {
                let rq: SearchEntityRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    5202,
                    &SearchEntityRs {
                        auto: rq.auto,
                        entity: self.grid.search_entities(rq.entity_type, rq.config_id),
                    },
                )
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
    use proto::slg::{BaseEntity, DispatchTroopRs, WorldEntityTypeDefine};
    use std::time::Duration;

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

    async fn wait_for_sector_troop_count(svc: &WorldServiceImpl, pos: i32, expected: usize) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        loop {
            if svc.sector_troop_count(pos) == expected {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "sector troop count did not reach {} for pos {}",
                expected,
                pos
            );
            tokio::task::yield_now().await;
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

        let rs = svc
            .dispatch(Request::new(request(50019, 42, &rq)))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 50020);
        let _: DispatchTroopRs = msg.get_payload().unwrap();

        wait_for_sector_troop_count(&svc, rq.pos, 1).await;
        let troops = svc.all_marching_troops();
        assert_eq!(troops.len(), 1);
        assert_eq!(troops[0].goal, Some(rq.pos));
        assert_eq!(
            troops[0].r#type,
            Some(crate::march::MARCH_TYPE_MINE_COLLECT)
        );
    }

    #[tokio::test]
    async fn get_player_troop_returns_started_marches() {
        let svc = service();
        svc.dispatch_troop(
            42,
            crate::map::grid::xy_to_pos(1, 1),
            crate::march::MARCH_TYPE_SCOUT,
        )
        .await
        .unwrap();

        let rs = svc
            .dispatch(Request::new(request(
                50031,
                42,
                &proto::slg::GetPlayerTroopRq::default(),
            )))
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
    async fn dispatch_scout_routes_into_sector() {
        let svc = service();
        let target = crate::map::grid::xy_to_pos(400, 400);
        let rq = DispatchScoutTroopRq {
            map: 1,
            pos: target,
            r#type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
        };

        let rs = svc
            .dispatch(Request::new(request(50039, 7, &rq)))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 50040);
        wait_for_sector_troop_count(&svc, target, 1).await;
        assert_eq!(svc.sector_troop_keys(target), vec![1]);
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

        let err = svc
            .dispatch(Request::new(request(50019, 42, &rq)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn troop_back_command_turns_marching_troop_home() {
        let svc = service();
        let origin = crate::map::grid::xy_to_pos(0, 0);
        let goal = crate::map::grid::xy_to_pos(100, 0);
        let now = crate::march::now_millis();

        svc.marching_mgr
            .start_march_at(
                BaseTroop {
                    key: 77,
                    r#type: Some(crate::march::MARCH_TYPE_ATK_PLAYER),
                    origin: Some(origin),
                    goal: Some(goal),
                    ..Default::default()
                },
                1.0,
                now - 5_000,
            )
            .unwrap();

        let rq = TroopBackCommandRq {
            key_id: 77,
            r#type: Some(1),
            cost_type: Some(1),
        };
        let rs = svc
            .dispatch(Request::new(request(50027, 42, &rq)))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 50028);
        let _: TroopBackCommandRs = msg.get_payload().unwrap();

        let troop = svc.marching_mgr.troops.get(&77).unwrap().base.clone();
        assert_eq!(troop.status, Some(crate::march::MARCH_STATUS_RETREAT));
        assert_eq!(troop.goal, Some(origin));
        assert_ne!(troop.origin, Some(origin));
    }

    #[tokio::test]
    async fn troop_accelerate_command_reduces_retreat_remaining_time() {
        let svc = service();
        let origin = crate::map::grid::xy_to_pos(10, 0);
        let goal = crate::map::grid::xy_to_pos(0, 0);
        let now = crate::march::now_millis();

        svc.marching_mgr.troops.insert(
            88,
            crate::march::MarchingTroop {
                base: BaseTroop {
                    key: 88,
                    r#type: Some(crate::march::MARCH_TYPE_ATK_PLAYER),
                    origin: Some(origin),
                    goal: Some(goal),
                    status: Some(crate::march::MARCH_STATUS_RETREAT),
                    start_time: Some(now),
                    end_time: Some(now + 10_000),
                    ..Default::default()
                },
                speed: 1.0,
            },
        );

        let rq = TroopAccelerateCommandRq {
            key_id: 88,
            r#type: Some(1),
            cost_type: Some(1),
        };
        let rs = svc
            .dispatch(Request::new(request(50029, 42, &rq)))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 50030);
        let _: TroopAccelerateCommandRs = msg.get_payload().unwrap();

        let troop = svc.marching_mgr.troops.get(&88).unwrap().base.clone();
        assert!(troop.end_time.unwrap() <= now + 5_500);
    }

    #[tokio::test]
    async fn troop_accelerate_command_rejects_non_retreat_troop() {
        let svc = service();
        svc.marching_mgr
            .start_march_at(
                BaseTroop {
                    key: 89,
                    r#type: Some(crate::march::MARCH_TYPE_ATK_PLAYER),
                    origin: Some(crate::map::grid::xy_to_pos(0, 0)),
                    goal: Some(crate::map::grid::xy_to_pos(20, 0)),
                    ..Default::default()
                },
                1.0,
                crate::march::now_millis(),
            )
            .unwrap();

        let rq = TroopAccelerateCommandRq {
            key_id: 89,
            r#type: Some(1),
            cost_type: Some(1),
        };
        let err = svc
            .dispatch(Request::new(request(50029, 42, &rq)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    }

    #[tokio::test]
    async fn search_entity_filters_by_type_and_config_and_echoes_auto() {
        let svc = service();
        let mine = BaseEntity {
            pos: crate::map::grid::xy_to_pos(10, 20),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            key_id: Some(1),
            conf_id: Some(301),
            ..Default::default()
        };
        let other_mine = BaseEntity {
            pos: crate::map::grid::xy_to_pos(11, 20),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            key_id: Some(2),
            conf_id: Some(302),
            ..Default::default()
        };
        let bandit = BaseEntity {
            pos: crate::map::grid::xy_to_pos(12, 20),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeBandit as i32),
            key_id: Some(3),
            conf_id: Some(301),
            ..Default::default()
        };
        svc.grid.upsert_entity(other_mine).unwrap();
        svc.grid.upsert_entity(bandit).unwrap();
        svc.grid.upsert_entity(mine.clone()).unwrap();

        let rq = SearchEntityRq {
            auto: Some(true),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            config_id: Some(301),
        };
        let rs = svc
            .dispatch(Request::new(request(5201, 42, &rq)))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, 5202);
        let body: SearchEntityRs = msg.get_payload().unwrap();

        assert_eq!(body.auto, Some(true));
        assert_eq!(body.entity, vec![mine]);
    }
}
