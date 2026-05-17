use dashmap::DashMap;
use proto::slg::world_service_server::WorldService;
use proto::slg::{
    BaseEntity, BaseMap, BasePlayerMapData, BaseTroop, DispatchPigeonTroopRq, DispatchRq,
    DispatchRs, DispatchScoutTroopRq, DispatchTroopRq, EnterWorldMapRq, GetAreaCityFirstKillInfoRq,
    GetAreaCityFirstKillInfoRs, GetAreaDetailsRq, GetAreaDetailsRs, GetBlockDetailsRq,
    GetBlockDetailsRs, GetEntityInfoRq, GetEntityInfoRs, GetFightDetailsRq, GetFightDetailsRs,
    GetFightInfoRq, GetFightInfoRs, GetFightListDetailsRq, GetFightListDetailsRs, GetMapDetailsRq,
    GetMapDetailsRs, GetNearestNonOwnCampCityRq, GetNearestNonOwnCampCityRs, GetPlayerTroopRs,
    GetTroopDetailsRq, GetTroopDetailsRs, GetTroopInfoRq, GetTroopInfoRs, JoinMapRequest,
    JoinMapResponse, LeaveWorldMapRs, MovePositionRq, MovePositionRs, RepatriateAssemblyTroopRq,
    RepatriateAssemblyTroopRs, RepatriateGarrisonTroopRq, RepatriateGarrisonTroopRs, RpcMsg,
    SearchEntityRq, SearchEntityRs, SelectPlayerGarrisonTroopRq, SelectPlayerGarrisonTroopRs,
    TroopAccelerateCommandRq, TroopAccelerateCommandRs, TroopBackCommandRq, TroopBackCommandRs,
};
use shared::msg::GameMessage;
use shared::static_config::WorldConfig;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

const DEFAULT_ENTITY_REFRESH_TIME_MS: u64 = 0;
const DEFAULT_BANDIT_CONF_ID: i32 = 201;
const DEFAULT_MINE_CONF_ID: i32 = 401;

pub struct WorldServiceImpl {
    grid: Arc<crate::map::grid::MapGrid>,
    entity_lifecycle: Mutex<crate::map::lifecycle::EntityLifecycleManager>,
    entity_spawn_rules: Vec<crate::map::lifecycle::EntitySpawnRule>,
    marching_mgr: Arc<crate::march::MarchingManager>,
    runtime: Arc<crate::runtime::WorldRuntime>,
    garrison_state: Arc<crate::garrison::GarrisonState>,
    assembly_state: Arc<crate::assembly::AssemblyState>,
    player_positions: DashMap<i64, i32>,
    aoi_senders: DashMap<i64, tokio::sync::mpsc::Sender<crate::map::aoi::AoiEvent>>,
    next_troop_key: AtomicI32,
}

impl WorldServiceImpl {
    pub fn new(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
    ) -> Self {
        Self::new_with_outbound(
            grid,
            marching_mgr,
            Arc::new(crate::outbound::InMemoryOutboundSink::new()),
        )
    }

    pub fn new_with_outbound(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
        outbound_sink: Arc<dyn crate::outbound::WorldOutboundSink>,
    ) -> Self {
        Self::new_with_outbound_config_and_spawn_rules(
            grid,
            marching_mgr,
            outbound_sink,
            Arc::new(WorldConfig::default()),
            default_entity_spawn_rules(),
        )
    }

    fn new_with_outbound_and_spawn_rules(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
        outbound_sink: Arc<dyn crate::outbound::WorldOutboundSink>,
        entity_spawn_rules: Vec<crate::map::lifecycle::EntitySpawnRule>,
    ) -> Self {
        Self::new_with_outbound_config_and_spawn_rules(
            grid,
            marching_mgr,
            outbound_sink,
            Arc::new(WorldConfig::default()),
            entity_spawn_rules,
        )
    }

    pub fn new_with_outbound_and_config(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
        outbound_sink: Arc<dyn crate::outbound::WorldOutboundSink>,
        world_config: Arc<WorldConfig>,
    ) -> Self {
        Self::new_with_outbound_config_and_spawn_rules(
            grid,
            marching_mgr,
            outbound_sink,
            world_config,
            default_entity_spawn_rules(),
        )
    }

    fn new_with_outbound_config_and_spawn_rules(
        grid: Arc<crate::map::grid::MapGrid>,
        marching_mgr: Arc<crate::march::MarchingManager>,
        outbound_sink: Arc<dyn crate::outbound::WorldOutboundSink>,
        world_config: Arc<WorldConfig>,
        entity_spawn_rules: Vec<crate::map::lifecycle::EntitySpawnRule>,
    ) -> Self {
        let runtime = Arc::new(crate::runtime::WorldRuntime::new_with_outbound_and_config(
            outbound_sink,
            world_config,
        ));
        let garrison_state = runtime.garrison_state();
        let assembly_state = runtime.assembly_state();
        let svc = Self {
            grid,
            entity_lifecycle: Mutex::new(crate::map::lifecycle::EntityLifecycleManager::new()),
            entity_spawn_rules,
            marching_mgr,
            runtime,
            garrison_state,
            assembly_state,
            player_positions: DashMap::new(),
            aoi_senders: DashMap::new(),
            next_troop_key: AtomicI32::new(1),
        };
        svc.refresh_default_entities_at(DEFAULT_ENTITY_REFRESH_TIME_MS)
            .expect("default world entity spawn rules must be valid");
        svc.runtime
            .sync_entity_snapshot_now(svc.grid.all_entities())
            .expect("default world entities must sync into sectors");
        svc
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

    fn refresh_default_entities_at(
        &self,
        now_ms: u64,
    ) -> Result<crate::map::lifecycle::EntityRefreshReport, Status> {
        let mut lifecycle = self
            .entity_lifecycle
            .lock()
            .map_err(|_| Status::internal("world entity lifecycle lock poisoned"))?;
        let report = lifecycle
            .refresh_at(&self.grid, &self.entity_spawn_rules, now_ms)
            .map_err(|e| {
                Status::internal(format!("refresh default world entities failed: {}", e))
            })?;
        for entity in &report.expired {
            self.runtime
                .sync_entity_remove_now(entity.pos)
                .map_err(|e| {
                    Status::internal(format!("sync world entity removal failed: {}", e))
                })?;
        }
        for entity in &report.spawned {
            self.runtime
                .sync_entity_upsert_now(entity.clone())
                .map_err(|e| Status::internal(format!("sync world entity failed: {}", e)))?;
        }
        Ok(report)
    }

    async fn dispatch_troop(
        &self,
        role_id: i64,
        target_pos: i32,
        troop_type: i32,
        formation_id: Option<i32>,
    ) -> Result<BaseTroop, Status> {
        if role_id <= 0 {
            return Err(Status::invalid_argument(format!(
                "invalid world role_id: {}",
                role_id
            )));
        }
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
        self.marching_mgr.set_troop_owner(troop.key, role_id);

        self.runtime
            .send_transfer_troop_with_formation(troop.clone(), formation_id)
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

    fn nearest_non_own_camp_city(&self, own_camp: i32, origin_pos: i32) -> Option<(i32, i32)> {
        self.grid
            .search_entities(
                Some(proto::slg::WorldEntityTypeDefine::EntityTypeCity as i32),
                None,
            )
            .into_iter()
            .filter(|entity| entity.camp != Some(own_camp))
            .min_by_key(|entity| {
                (
                    map_distance_squared(origin_pos, entity.pos),
                    entity.pos,
                    entity.key_id.unwrap_or_default(),
                )
            })
            .map(|entity| (entity.key_id.unwrap_or_default(), entity.pos))
    }

    fn move_player_city(&self, role_id: i64, pos: i32) -> Result<BasePlayerMapData, Status> {
        if !crate::map::grid::is_valid_pos(pos) {
            return Err(Status::invalid_argument(format!(
                "invalid world player position: {}",
                pos
            )));
        }

        let old_pos = self.player_positions.insert(role_id, pos);
        if let Some(old_pos) = old_pos {
            if old_pos != pos {
                self.grid.remove_entity(old_pos);
                self.runtime.sync_entity_remove_now(old_pos).map_err(|e| {
                    Status::internal(format!("sync old player city removal failed: {}", e))
                })?;
            }
        }

        let entity = BaseEntity {
            pos,
            entity_type: Some(proto::slg::WorldEntityTypeDefine::EntityTypePlayer as i32),
            key_id: i32::try_from(role_id).ok(),
            ..Default::default()
        };
        self.grid
            .upsert_entity(entity.clone())
            .map_err(|e| Status::failed_precondition(format!("move player city failed: {}", e)))?;
        self.runtime
            .sync_entity_upsert_now(entity)
            .map_err(|e| Status::internal(format!("sync player city failed: {}", e)))?;
        self.update_aoi_subscription(role_id, old_pos, pos);

        Ok(BasePlayerMapData {
            map: 1,
            role_id,
            pos,
            ..Default::default()
        })
    }

    fn enter_world_map(&self, role_id: i64, map_id: i32) -> Result<BasePlayerMapData, Status> {
        let pos = self
            .player_positions
            .get(&role_id)
            .map(|entry| *entry.value())
            .unwrap_or(0);
        let mut player_data = self.move_player_city(role_id, pos)?;
        player_data.map = map_id;
        Ok(player_data)
    }

    fn leave_world_map(&self, role_id: i64) -> Result<(), Status> {
        let Some((_, pos)) = self.player_positions.remove(&role_id) else {
            self.aoi_senders.remove(&role_id);
            return Ok(());
        };

        self.runtime.aoi_manager().unsubscribe_area(pos, role_id);
        self.aoi_senders.remove(&role_id);
        self.grid.remove_entity(pos);
        self.runtime
            .sync_entity_remove_now(pos)
            .map_err(|e| Status::internal(format!("sync player leave removal failed: {}", e)))
    }

    fn update_aoi_subscription(&self, role_id: i64, old_pos: Option<i32>, new_pos: i32) {
        let tx = self.aoi_sender_for_role(role_id);
        let aoi = self.runtime.aoi_manager();
        match old_pos {
            Some(old_pos) => aoi.move_subscription(old_pos, new_pos, role_id, tx),
            None => aoi.subscribe_area(new_pos, role_id, tx),
        }
    }

    fn aoi_sender_for_role(
        &self,
        role_id: i64,
    ) -> tokio::sync::mpsc::Sender<crate::map::aoi::AoiEvent> {
        if let Some(sender) = self.aoi_senders.get(&role_id) {
            return sender.clone();
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(128);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });
        self.aoi_senders.insert(role_id, tx.clone());
        tx
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

    #[cfg(test)]
    fn sector_entity_positions(&self, pos: i32) -> Vec<i32> {
        self.runtime
            .sector_entity_positions(crate::runtime::WorldRuntime::sector_id_for_pos(pos))
    }

    #[cfg(test)]
    fn aoi_subscription_count_for_pos(&self, pos: i32, role_id: i64) -> usize {
        crate::map::grid::MapGrid::get_view_grid_ids(pos)
            .into_iter()
            .filter(|gid| {
                self.runtime
                    .aoi_manager()
                    .subscribers(*gid)
                    .contains(&role_id)
            })
            .count()
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
                let player_data = self.enter_world_map(req.role_id, rq.map_id)?;
                self.response(
                    50004,
                    &proto::slg::EnterWorldMapRs {
                        player_data: Some(player_data),
                    },
                )
            }
            50005 => {
                let _rq: proto::slg::LeaveWorldMapRq = self.decode_payload(req.cmd, req.payload)?;
                self.leave_world_map(req.role_id)?;
                self.response(50006, &LeaveWorldMapRs::default())
            }
            50007 => {
                let rq: GetAreaDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50008,
                    &GetAreaDetailsRs {
                        entity: self.grid.entities_in_area(rq.area),
                    },
                )
            }
            50009 => {
                let rq: GetBlockDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    50010,
                    &GetBlockDetailsRs {
                        entity: self.grid.entities_in_blocks(&rq.block),
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
                let player_data = self.move_player_city(req.role_id, rq.pos.unwrap_or(0))?;
                self.response(
                    50014,
                    &MovePositionRs {
                        player_data: Some(BasePlayerMapData {
                            map: rq.map,
                            ..player_data
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
                    .dispatch_troop(
                        req.role_id,
                        rq.pos,
                        Self::troop_type_for_entity(rq.r#type),
                        Some(rq.formation_id),
                    )
                    .await?;
                self.response(50020, &proto::slg::DispatchTroopRs::default())
            }
            50023 => {
                let rq: proto::slg::DeclareFightRq = self.decode_payload(req.cmd, req.payload)?;
                if !crate::map::grid::is_valid_pos(rq.pos) {
                    return Err(Status::invalid_argument(format!(
                        "invalid world fight position: {}",
                        rq.pos
                    )));
                }
                Err(Status::failed_precondition(format!(
                    "declare fight is unavailable until battle service is integrated: pos={}, fight_type={:?}",
                    rq.pos, rq.fight_type
                )))
            }
            50025 => {
                let rq: proto::slg::JoinTheFightRq = self.decode_payload(req.cmd, req.payload)?;
                if rq.fight_id <= 0 {
                    return Err(Status::invalid_argument(format!(
                        "invalid world fight id: {}",
                        rq.fight_id
                    )));
                }
                Err(Status::failed_precondition(format!(
                    "join fight is unavailable until battle service is integrated: fight_id={}",
                    rq.fight_id
                )))
            }
            50027 => {
                let rq: TroopBackCommandRq = self.decode_payload(req.cmd, req.payload)?;
                let troop = self
                    .marching_mgr
                    .recall_troop(rq.key_id, rq.r#type)
                    .map_err(|e| {
                        Status::failed_precondition(format!("recall troop failed: {}", e))
                    })?;
                self.runtime.sync_troop_update(troop).await.map_err(|e| {
                    Status::failed_precondition(format!("sector recall sync failed: {}", e))
                })?;
                self.response(50028, &TroopBackCommandRs::default())
            }
            50029 => {
                let rq: TroopAccelerateCommandRq = self.decode_payload(req.cmd, req.payload)?;
                let troop = self
                    .marching_mgr
                    .accelerate_troop(rq.key_id, rq.r#type)
                    .map_err(|e| {
                        Status::failed_precondition(format!("accelerate troop failed: {}", e))
                    })?;
                self.runtime.sync_troop_update(troop).await.map_err(|e| {
                    Status::failed_precondition(format!("sector accelerate sync failed: {}", e))
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
                    .dispatch_troop(
                        req.role_id,
                        rq.pos,
                        crate::march::MARCH_TYPE_INTEL_TASK,
                        None,
                    )
                    .await?;
                self.response(50038, &proto::slg::DispatchPigeonTroopRs::default())
            }
            50039 => {
                let rq: DispatchScoutTroopRq = self.decode_payload(req.cmd, req.payload)?;
                let _troop = self
                    .dispatch_troop(req.role_id, rq.pos, crate::march::MARCH_TYPE_SCOUT, None)
                    .await?;
                self.response(50040, &proto::slg::DispatchScoutTroopRs::default())
            }
            5121 => {
                let rq: SelectPlayerGarrisonTroopRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    5122,
                    &SelectPlayerGarrisonTroopRs {
                        garrison_troop: self.garrison_state.list(rq.pos),
                    },
                )
            }
            5123 => {
                let rq: RepatriateGarrisonTroopRq = self.decode_payload(req.cmd, req.payload)?;
                if rq.troop_key.unwrap_or_default() == 0 {
                    self.garrison_state.repatriate_all();
                } else {
                    self.garrison_state
                        .repatriate_one(rq.troop_key.unwrap_or_default())
                        .map_err(|e| {
                            Status::failed_precondition(format!(
                                "repatriate garrison troop failed: {}",
                                e
                            ))
                        })?;
                }
                self.response(5124, &RepatriateGarrisonTroopRs::default())
            }
            5141 => {
                let _rq: GetFightListDetailsRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    5142,
                    &GetFightListDetailsRs {
                        base_fight: Vec::new(),
                    },
                )
            }
            5143 => {
                let _rq: GetFightInfoRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(5144, &GetFightInfoRs { base_fight: None })
            }
            5145 => {
                let rq: RepatriateAssemblyTroopRq = self.decode_payload(req.cmd, req.payload)?;
                self.assembly_state
                    .repatriate(rq.assembly_id.unwrap_or_default(), rq.troop_key)
                    .map_err(|e| {
                        Status::failed_precondition(format!(
                            "repatriate assembly troop failed: {}",
                            e
                        ))
                    })?;
                self.response(5146, &RepatriateAssemblyTroopRs::default())
            }
            5147 => {
                let rq: proto::slg::CancelAssemblyRq = self.decode_payload(req.cmd, req.payload)?;
                self.assembly_state
                    .cancel(rq.assembly_id.unwrap_or_default())
                    .map_err(|e| {
                        Status::failed_precondition(format!("cancel assembly failed: {}", e))
                    })?;
                self.response(5148, &proto::slg::CancelAssemblyRs::default())
            }
            5161 => {
                let _rq: proto::slg::CitySupplementArmyRq =
                    self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    5162,
                    &proto::slg::CitySupplementArmyRs {
                        npc_info: Vec::new(),
                    },
                )
            }
            5163 => {
                let _rq: GetAreaCityFirstKillInfoRq = self.decode_payload(req.cmd, req.payload)?;
                self.response(
                    5164,
                    &GetAreaCityFirstKillInfoRs {
                        first_defeat_team: Vec::new(),
                    },
                )
            }
            5165 => {
                let _rq: GetNearestNonOwnCampCityRq = self.decode_payload(req.cmd, req.payload)?;
                let nearest = self.nearest_non_own_camp_city(0, 0);
                self.response(
                    5166,
                    &GetNearestNonOwnCampCityRs {
                        city_id: nearest.map(|(city_id, _)| city_id),
                        city_pos: nearest.map(|(_, city_pos)| city_pos),
                    },
                )
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

fn default_entity_spawn_rules() -> Vec<crate::map::lifecycle::EntitySpawnRule> {
    vec![
        crate::map::lifecycle::EntitySpawnRule::at_positions(
            proto::slg::WorldEntityTypeDefine::EntityTypeBandit as i32,
            DEFAULT_BANDIT_CONF_ID,
            1,
            vec![crate::map::grid::xy_to_pos(100, 100)],
            None,
        ),
        crate::map::lifecycle::EntitySpawnRule::at_positions(
            proto::slg::WorldEntityTypeDefine::EntityTypeMine as i32,
            DEFAULT_MINE_CONF_ID,
            1,
            vec![crate::map::grid::xy_to_pos(101, 100)],
            None,
        ),
    ]
}

fn map_distance_squared(left: i32, right: i32) -> i32 {
    let (left_x, left_y) = crate::map::grid::pos_to_xy(left);
    let (right_x, right_y) = crate::map::grid::pos_to_xy(right);
    (left_x - right_x).pow(2) + (left_y - right_y).pow(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::slg::{BaseEntity, DispatchTroopRs, GarrisonTroop, WorldEntityTypeDefine};
    use std::time::Duration;

    fn service() -> WorldServiceImpl {
        WorldServiceImpl::new(
            Arc::new(crate::map::grid::MapGrid::new()),
            Arc::new(crate::march::MarchingManager::new()),
        )
    }

    fn service_with_spawn_rules(
        rules: Vec<crate::map::lifecycle::EntitySpawnRule>,
    ) -> WorldServiceImpl {
        WorldServiceImpl::new_with_outbound_and_spawn_rules(
            Arc::new(crate::map::grid::MapGrid::new()),
            Arc::new(crate::march::MarchingManager::new()),
            Arc::new(crate::outbound::InMemoryOutboundSink::new()),
            rules,
        )
    }

    fn request<T: prost::Message>(cmd: i32, role_id: i64, body: &T) -> DispatchRq {
        DispatchRq {
            role_id,
            cmd,
            payload: GameMessage::build_response(cmd, body).unwrap(),
        }
    }

    async fn dispatch_body<T, R>(svc: &WorldServiceImpl, cmd: i32, response_cmd: i32, body: &T) -> R
    where
        T: prost::Message,
        R: prost::Message + Default,
    {
        let rs = svc
            .dispatch(Request::new(request(cmd, 42, body)))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(rs.code, 0);
        let msg = GameMessage::decode(rs.payload).unwrap();
        assert_eq!(msg.base.cmd, response_cmd);
        msg.get_payload().unwrap()
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

    async fn wait_for_sector_entity_positions(
        svc: &WorldServiceImpl,
        pos: i32,
        expected: Vec<i32>,
    ) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        loop {
            if svc.sector_entity_positions(pos) == expected {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "sector entity positions did not reach {:?} for pos {}",
                expected,
                pos
            );
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn enter_move_and_leave_world_map_manage_player_aoi_subscription() {
        let svc = service();
        let role_id = 42;
        let enter: proto::slg::EnterWorldMapRs =
            dispatch_body(&svc, 50003, 50004, &EnterWorldMapRq { map_id: 1 }).await;
        let player_data = enter.player_data.unwrap();

        assert_eq!(player_data.role_id, role_id);
        assert_eq!(player_data.pos, 0);
        assert_eq!(
            svc.aoi_subscription_count_for_pos(player_data.pos, role_id),
            crate::map::grid::MapGrid::get_view_grid_ids(player_data.pos).len()
        );
        assert!(svc.grid.get_entity(player_data.pos).is_some());

        let new_pos = crate::map::grid::xy_to_pos(400, 400);
        let moved: MovePositionRs = dispatch_body(
            &svc,
            50013,
            50014,
            &MovePositionRq {
                map: 1,
                r#type: 2,
                pos: Some(new_pos),
            },
        )
        .await;
        let moved_player_data = moved.player_data.unwrap();

        assert_eq!(moved_player_data.role_id, role_id);
        assert_eq!(moved_player_data.pos, new_pos);
        assert!(svc.grid.get_entity(player_data.pos).is_none());
        assert!(svc.grid.get_entity(new_pos).is_some());
        assert_eq!(
            svc.aoi_subscription_count_for_pos(player_data.pos, role_id),
            0
        );
        assert_eq!(
            svc.aoi_subscription_count_for_pos(new_pos, role_id),
            crate::map::grid::MapGrid::get_view_grid_ids(new_pos).len()
        );

        let _leave: LeaveWorldMapRs =
            dispatch_body(&svc, 50005, 50006, &proto::slg::LeaveWorldMapRq::default()).await;

        assert!(svc.player_positions.get(&role_id).is_none());
        assert_eq!(svc.aoi_subscription_count_for_pos(new_pos, role_id), 0);
        assert!(svc.grid.get_entity(new_pos).is_none());
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
            None,
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
    async fn dispatch_troop_registers_owner_for_outbound_resolution() {
        let svc = service();
        let role_id = 900_001;
        let target = crate::map::grid::xy_to_pos(50, 50);

        let troop = svc
            .dispatch_troop(role_id, target, crate::march::MARCH_TYPE_SCOUT, Some(7))
            .await
            .unwrap();

        assert_eq!(svc.marching_mgr.troop_owner(troop.key), Some(role_id));
    }

    #[tokio::test]
    async fn move_position_updates_player_city_entity() {
        let svc = service();
        let first_pos = crate::map::grid::xy_to_pos(400, 400);
        let second_pos = crate::map::grid::xy_to_pos(460, 400);

        let first: MovePositionRs = dispatch_body(
            &svc,
            50013,
            50014,
            &MovePositionRq {
                map: 1,
                r#type: 2,
                pos: Some(first_pos),
            },
        )
        .await;
        assert_eq!(
            first.player_data.as_ref().map(|data| data.pos),
            Some(first_pos)
        );
        wait_for_sector_entity_positions(&svc, first_pos, vec![first_pos]).await;

        let second: MovePositionRs = dispatch_body(
            &svc,
            50013,
            50014,
            &MovePositionRq {
                map: 1,
                r#type: 2,
                pos: Some(second_pos),
            },
        )
        .await;
        assert_eq!(
            second.player_data.as_ref().map(|data| data.pos),
            Some(second_pos)
        );

        let old: GetEntityInfoRs = dispatch_body(
            &svc,
            50011,
            50012,
            &GetEntityInfoRq {
                map: 1,
                pos: first_pos,
            },
        )
        .await;
        assert!(old.entity.is_none());

        let new: GetEntityInfoRs = dispatch_body(
            &svc,
            50011,
            50012,
            &GetEntityInfoRq {
                map: 1,
                pos: second_pos,
            },
        )
        .await;
        let entity = new.entity.unwrap();
        assert_eq!(entity.pos, second_pos);
        assert_eq!(
            entity.entity_type,
            Some(WorldEntityTypeDefine::EntityTypePlayer as i32)
        );
        assert_eq!(entity.key_id, Some(42));

        wait_for_sector_entity_positions(&svc, second_pos, vec![second_pos]).await;
    }

    #[tokio::test]
    async fn fight_commands_decode_and_report_battle_integration_gap() {
        let svc = service();

        let declare = proto::slg::DeclareFightRq {
            pos: crate::map::grid::xy_to_pos(10, 10),
            fight_type: Some(1),
        };
        let err = svc
            .dispatch(Request::new(request(50023, 42, &declare)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::FailedPrecondition);
        assert!(err.message().contains("battle service"));

        let join = proto::slg::JoinTheFightRq { fight_id: 1 };
        let err = svc
            .dispatch(Request::new(request(50025, 42, &join)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::FailedPrecondition);
        assert!(err.message().contains("battle service"));
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
    async fn service_initialization_refreshes_default_entities_for_search() {
        let svc = service();

        let body: SearchEntityRs = dispatch_body(
            &svc,
            5201,
            5202,
            &SearchEntityRq {
                auto: Some(false),
                entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
                config_id: Some(DEFAULT_MINE_CONF_ID),
            },
        )
        .await;

        assert_eq!(body.entity.len(), 1);
        assert_eq!(body.entity[0].pos, crate::map::grid::xy_to_pos(101, 100));
        assert_eq!(
            body.entity[0].entity_type,
            Some(WorldEntityTypeDefine::EntityTypeMine as i32)
        );
        assert_eq!(body.entity[0].conf_id, Some(DEFAULT_MINE_CONF_ID));

        let area: GetAreaDetailsRs =
            dispatch_body(&svc, 50007, 50008, &GetAreaDetailsRq { map: 1, area: None }).await;
        assert_eq!(area.entity.len(), 2);
        assert!(area
            .entity
            .iter()
            .any(|entity| entity.conf_id == Some(DEFAULT_BANDIT_CONF_ID)));
        assert!(area
            .entity
            .iter()
            .any(|entity| entity.conf_id == Some(DEFAULT_MINE_CONF_ID)));

        wait_for_sector_entity_positions(
            &svc,
            crate::map::grid::xy_to_pos(100, 100),
            vec![
                crate::map::grid::xy_to_pos(100, 100),
                crate::map::grid::xy_to_pos(101, 100),
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn default_entity_refresh_does_not_duplicate_existing_entities() {
        let svc = service();
        let before = svc.grid.all_entities();

        let report = svc.refresh_default_entities_at(1_000).unwrap();
        let after = svc.grid.all_entities();

        assert!(report.expired.is_empty());
        assert!(report.spawned.is_empty());
        assert_eq!(after, before);
    }

    #[tokio::test]
    async fn default_entity_refresh_expires_then_respawns_configured_ttl_entities() {
        let pos = crate::map::grid::xy_to_pos(120, 120);
        let svc =
            service_with_spawn_rules(vec![crate::map::lifecycle::EntitySpawnRule::at_positions(
                WorldEntityTypeDefine::EntityTypeMine as i32,
                DEFAULT_MINE_CONF_ID,
                1,
                vec![pos],
                Some(500),
            )]);

        assert_eq!(svc.grid.search_entities(None, None).len(), 1);

        let early = svc.refresh_default_entities_at(499).unwrap();
        assert!(early.expired.is_empty());
        assert!(early.spawned.is_empty());

        let report = svc.refresh_default_entities_at(500).unwrap();

        assert_eq!(
            report
                .expired
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            vec![pos]
        );
        assert_eq!(
            report
                .spawned
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            vec![pos]
        );
        assert_eq!(svc.grid.search_entities(None, None).len(), 1);
        wait_for_sector_entity_positions(&svc, pos, vec![pos]).await;
    }

    #[tokio::test]
    async fn default_entity_refresh_adopts_recovered_ttl_entities_before_expiring() {
        let pos = crate::map::grid::xy_to_pos(121, 120);
        let mut svc = service_with_spawn_rules(Vec::new());
        svc.entity_spawn_rules = vec![crate::map::lifecycle::EntitySpawnRule::at_positions(
            WorldEntityTypeDefine::EntityTypeMine as i32,
            DEFAULT_MINE_CONF_ID,
            1,
            vec![pos],
            Some(500),
        )];
        svc.grid
            .upsert_entity(BaseEntity {
                pos,
                entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
                key_id: Some(9_001),
                conf_id: Some(DEFAULT_MINE_CONF_ID),
                ..Default::default()
            })
            .unwrap();

        let adopted = svc.refresh_default_entities_at(1_000).unwrap();
        assert!(adopted.expired.is_empty());
        assert!(adopted.spawned.is_empty());

        let expired = svc.refresh_default_entities_at(1_500).unwrap();
        assert_eq!(
            expired
                .expired
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            vec![pos]
        );
        assert_eq!(
            expired
                .spawned
                .iter()
                .map(|entity| entity.pos)
                .collect::<Vec<_>>(),
            vec![pos]
        );
    }

    #[tokio::test]
    async fn area_and_block_details_filter_entities() {
        let svc = service();
        let target = BaseEntity {
            pos: crate::map::grid::xy_to_pos(400, 400),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            key_id: Some(9001),
            ..Default::default()
        };
        let same_area_other_block = BaseEntity {
            pos: crate::map::grid::xy_to_pos(460, 400),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeMine as i32),
            key_id: Some(9002),
            ..Default::default()
        };
        let other_area = BaseEntity {
            pos: crate::map::grid::xy_to_pos(900, 900),
            entity_type: Some(WorldEntityTypeDefine::EntityTypeBandit as i32),
            key_id: Some(9003),
            ..Default::default()
        };
        svc.grid.upsert_entity(other_area).unwrap();
        svc.grid
            .upsert_entity(same_area_other_block.clone())
            .unwrap();
        svc.grid.upsert_entity(target.clone()).unwrap();

        let area: GetAreaDetailsRs = dispatch_body(
            &svc,
            50007,
            50008,
            &GetAreaDetailsRq {
                map: 1,
                area: Some(crate::map::grid::pos_to_sector_id(target.pos)),
            },
        )
        .await;
        assert_eq!(area.entity, vec![target.clone(), same_area_other_block]);

        let block: GetBlockDetailsRs = dispatch_body(
            &svc,
            50009,
            50010,
            &GetBlockDetailsRq {
                map: 1,
                block: vec![crate::map::grid::pos_to_grid(target.pos)],
            },
        )
        .await;
        assert_eq!(block.entity, vec![target]);
        assert_eq!(
            block.block,
            vec![crate::map::grid::pos_to_grid(crate::map::grid::xy_to_pos(
                400, 400
            ))]
        );

        let all_blocks: GetBlockDetailsRs = dispatch_body(
            &svc,
            50009,
            50010,
            &GetBlockDetailsRq {
                map: 1,
                block: Vec::new(),
            },
        )
        .await;
        assert_eq!(all_blocks.entity.len(), svc.grid.all_entities().len());
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

    #[tokio::test]
    async fn select_garrison_troop_returns_matching_garrison_marches() {
        let svc = service();
        let target = crate::map::grid::xy_to_pos(20, 20);
        svc.garrison_state
            .place(
                target,
                GarrisonTroop {
                    troop_key_id: Some(66),
                    end_time: Some(12345),
                    ..Default::default()
                },
            )
            .unwrap();
        svc.garrison_state
            .place(
                crate::map::grid::xy_to_pos(30, 30),
                GarrisonTroop {
                    troop_key_id: Some(67),
                    ..Default::default()
                },
            )
            .unwrap();

        let body: SelectPlayerGarrisonTroopRs = dispatch_body(
            &svc,
            5121,
            5122,
            &SelectPlayerGarrisonTroopRq { pos: Some(target) },
        )
        .await;

        assert_eq!(body.garrison_troop.len(), 1);
        assert_eq!(body.garrison_troop[0].troop_key_id, Some(66));
        assert_eq!(body.garrison_troop[0].end_time, Some(12345));
    }

    #[tokio::test]
    async fn remaining_world_business_commands_return_compatible_empty_responses() {
        let svc = service();
        svc.assembly_state.create(1, 66).unwrap();
        svc.assembly_state.add_troop(1, 67).unwrap();

        let _: RepatriateGarrisonTroopRs = dispatch_body(
            &svc,
            5123,
            5124,
            &RepatriateGarrisonTroopRq { troop_key: Some(0) },
        )
        .await;

        let fights: GetFightListDetailsRs =
            dispatch_body(&svc, 5141, 5142, &GetFightListDetailsRq::default()).await;
        assert!(fights.base_fight.is_empty());

        let fight: GetFightInfoRs =
            dispatch_body(&svc, 5143, 5144, &GetFightInfoRq { fight_id: Some(1) }).await;
        assert!(fight.base_fight.is_none());

        let _: RepatriateAssemblyTroopRs = dispatch_body(
            &svc,
            5145,
            5146,
            &RepatriateAssemblyTroopRq {
                assembly_id: Some(1),
                troop_key: 66,
            },
        )
        .await;
        assert_eq!(svc.assembly_state.snapshot(1).unwrap().troop_keys, vec![67]);

        let _: proto::slg::CancelAssemblyRs = dispatch_body(
            &svc,
            5147,
            5148,
            &proto::slg::CancelAssemblyRq {
                assembly_id: Some(1),
            },
        )
        .await;

        let city: proto::slg::CitySupplementArmyRs = dispatch_body(
            &svc,
            5161,
            5162,
            &proto::slg::CitySupplementArmyRq { city_id: Some(1) },
        )
        .await;
        assert!(city.npc_info.is_empty());

        let first_kill: GetAreaCityFirstKillInfoRs = dispatch_body(
            &svc,
            5163,
            5164,
            &GetAreaCityFirstKillInfoRq {
                area_id: 1,
                city_type: None,
            },
        )
        .await;
        assert!(first_kill.first_defeat_team.is_empty());
    }

    #[tokio::test]
    async fn nearest_non_own_camp_city_returns_closest_matching_city() {
        let svc = service();
        svc.grid
            .upsert_entity(BaseEntity {
                pos: crate::map::grid::xy_to_pos(1, 0),
                entity_type: Some(WorldEntityTypeDefine::EntityTypeCity as i32),
                key_id: Some(100),
                camp: Some(0),
                ..Default::default()
            })
            .unwrap();
        svc.grid
            .upsert_entity(BaseEntity {
                pos: crate::map::grid::xy_to_pos(10, 0),
                entity_type: Some(WorldEntityTypeDefine::EntityTypeCity as i32),
                key_id: Some(200),
                camp: Some(1),
                ..Default::default()
            })
            .unwrap();
        svc.grid
            .upsert_entity(BaseEntity {
                pos: crate::map::grid::xy_to_pos(20, 0),
                entity_type: Some(WorldEntityTypeDefine::EntityTypeCity as i32),
                key_id: Some(300),
                camp: Some(2),
                ..Default::default()
            })
            .unwrap();

        let body: GetNearestNonOwnCampCityRs =
            dispatch_body(&svc, 5165, 5166, &GetNearestNonOwnCampCityRq::default()).await;

        assert_eq!(body.city_id, Some(200));
        assert_eq!(body.city_pos, Some(crate::map::grid::xy_to_pos(10, 0)));
    }
}
