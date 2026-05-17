use anyhow::Result;
use prost::Message;
use proto::slg::{AwardPb, FunctionClientBase, ShopBuyRq, ShopBuyRs};
use shared::event::GameEvent;
use shared::msg::ToFunctionClientBaseBytes;
use shared::persistence::{PlayerDataRow, SaveEntry};
use shared::static_config::StaticConfig;
use std::sync::Arc;
use tracing::{error, warn};

use crate::actors::player_actor::PlayerActor;
use crate::systems::PlayerSystem;

const LORD_RESOURCE_AWARD_TYPE: i32 = 1;
const LEGACY_LORD_RESOURCE_AWARD_TYPE: i32 = 2;
const ITEM_AWARD_TYPE: i32 = 4;
const LORD_RESOURCE_DIAMOND: i32 = 1;
const LORD_RESOURCE_GOLD: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeCommandTarget {
    Mission,
    Building,
    Hero,
    Backpack,
    Technology,
    Equip,
    Mail,
    Chat,
    Vip,
    Shop,
    Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HomeCommandRoute {
    pub start: u32,
    pub end: u32,
    pub target: HomeCommandTarget,
    pub label: &'static str,
}

impl HomeCommandRoute {
    pub const fn new(start: u32, end: u32, target: HomeCommandTarget, label: &'static str) -> Self {
        Self {
            start,
            end,
            target,
            label,
        }
    }

    pub fn contains(&self, cmd: u32) -> bool {
        self.start <= cmd && cmd <= self.end
    }
}

/// Home command routing table.
///
/// Add new Home systems here rather than growing `PlayerActor::handle_game_command`.
/// Routes are matched in declaration order, so future narrow ranges (for example
/// Shop/VIP/Chat/Rank/GM) should be inserted before broader catch-all ranges such
/// as Activity when their command IDs overlap.
pub const HOME_COMMAND_ROUTES: &[HomeCommandRoute] = &[
    HomeCommandRoute::new(1101, 1200, HomeCommandTarget::Mission, "mission"),
    HomeCommandRoute::new(1501, 1603, HomeCommandTarget::Building, "building"),
    HomeCommandRoute::new(2001, 2500, HomeCommandTarget::Hero, "hero"),
    HomeCommandRoute::new(4001, 4004, HomeCommandTarget::Backpack, "backpack"),
    HomeCommandRoute::new(4201, 4208, HomeCommandTarget::Technology, "technology"),
    HomeCommandRoute::new(4801, 5100, HomeCommandTarget::Equip, "equip"),
    HomeCommandRoute::new(6001, 6026, HomeCommandTarget::Mail, "mail"),
    HomeCommandRoute::new(7601, 7650, HomeCommandTarget::Chat, "chat"),
    HomeCommandRoute::new(7401, 7500, HomeCommandTarget::Vip, "vip"),
    HomeCommandRoute::new(7651, 7700, HomeCommandTarget::Shop, "shop"),
    HomeCommandRoute::new(8001, 10000, HomeCommandTarget::Activity, "activity"),
];

pub fn route_home_command(cmd: u32) -> Option<HomeCommandTarget> {
    HOME_COMMAND_ROUTES
        .iter()
        .find(|route| route.contains(cmd))
        .map(|route| route.target)
}

pub struct SystemCommandResult {
    pub response_payload: Vec<u8>,
    pub events: Vec<GameEvent>,
}

/// Registry operations for systems owned by `PlayerActor`.
///
/// This keeps persistence/lifecycle/client-data/command routing in one Home-side
/// module. New per-player systems should add their field to `PlayerActor`, then
/// register their persistence/lifecycle/client-data and command route here.
pub trait HomeSystemRegistry {
    fn load_registered_systems_from_row(&mut self, row: &PlayerDataRow);

    fn on_registered_systems_login(&mut self);

    fn tick_registered_systems(&mut self);

    fn collect_registered_save_entries(
        &mut self,
        force_all: bool,
    ) -> (Vec<SaveEntry>, Vec<&'static str>);

    fn clear_registered_system_dirty(&mut self, column: &str) -> bool;

    fn append_registered_function_bases(&self, function_base: &mut Vec<FunctionClientBase>);

    fn dispatch_registered_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<SystemCommandResult>;
}

impl HomeSystemRegistry for PlayerActor {
    fn load_registered_systems_from_row(&mut self, row: &PlayerDataRow) {
        let systems: Vec<(&mut dyn PlayerSystem, Option<&Vec<u8>>)> = vec![
            (&mut self.activity_system, row.activity_func.as_ref()),
            (&mut self.hero_system, row.hero_func.as_ref()),
            (&mut self.backpack_system, row.backpack_func.as_ref()),
            (&mut self.building_system, row.sim_func.as_ref()),
            (&mut self.tech_system, row.technology_func.as_ref()),
            (&mut self.equip_system, row.equip_func.as_ref()),
            (&mut self.mail_system, row.mail_func.as_ref()),
            (&mut self.chat_system, row.chat_func.as_ref()),
            (&mut self.mission_system, row.mission_func.as_ref()),
            (&mut self.skin_system, row.skin_func.as_ref()),
            (&mut self.shop_system, row.shop_func.as_ref()),
            (&mut self.vip_system, row.vip_func.as_ref()),
        ];

        let shop_blob_loaded = row
            .shop_func
            .as_ref()
            .map(|data| !data.is_empty())
            .unwrap_or(false);
        let vip_blob_loaded = row
            .vip_func
            .as_ref()
            .map(|data| !data.is_empty())
            .unwrap_or(false);

        for (system, data_opt) in systems {
            if let Some(data) = data_opt {
                if !data.is_empty() {
                    if let Err(e) = system.load_from_bin(data) {
                        warn!(
                            role_id = self.role_id,
                            col = system.column_name(),
                            "Failed to deserialize, using default: {}",
                            e
                        );
                    }
                }
            }
        }

        if !shop_blob_loaded {
            self.shop_system
                .ensure_configured(&self.current_config.shop);
        }
        if !vip_blob_loaded {
            if let Some(lord) = &self.lord {
                self.vip_system.sync_from_lord_row(lord);
            }
        }
    }

    fn on_registered_systems_login(&mut self) {
        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.hero_system,
            &mut self.backpack_system,
            &mut self.building_system,
            &mut self.tech_system,
            &mut self.equip_system,
            &mut self.mail_system,
            &mut self.chat_system,
            &mut self.mission_system,
            &mut self.skin_system,
            &mut self.shop_system,
            &mut self.vip_system,
        ];

        for system in systems {
            system.on_login();
        }
    }

    fn tick_registered_systems(&mut self) {
        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.hero_system,
            &mut self.backpack_system,
            &mut self.building_system,
            &mut self.tech_system,
            &mut self.equip_system,
            &mut self.mail_system,
            &mut self.chat_system,
            &mut self.mission_system,
            &mut self.skin_system,
            &mut self.shop_system,
            &mut self.vip_system,
        ];

        for system in systems {
            system.tick();
        }
    }

    fn collect_registered_save_entries(
        &mut self,
        force_all: bool,
    ) -> (Vec<SaveEntry>, Vec<&'static str>) {
        let mut entries: Vec<SaveEntry> = Vec::new();
        let mut saved_columns: Vec<&'static str> = Vec::new();

        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.hero_system,
            &mut self.backpack_system,
            &mut self.building_system,
            &mut self.tech_system,
            &mut self.equip_system,
            &mut self.mail_system,
            &mut self.chat_system,
            &mut self.mission_system,
            &mut self.skin_system,
            &mut self.shop_system,
            &mut self.vip_system,
        ];

        for system in systems {
            if force_all || system.is_dirty() {
                match system.save_to_bin() {
                    Ok(data) => {
                        let column = system.column_name();
                        entries.push(SaveEntry { column, data });
                        saved_columns.push(column);
                    }
                    Err(e) => {
                        error!(
                            role_id = self.role_id,
                            col = system.column_name(),
                            "Serialize failed: {}",
                            e
                        );
                    }
                }
            }
        }

        (entries, saved_columns)
    }

    fn clear_registered_system_dirty(&mut self, column: &str) -> bool {
        let systems: Vec<&mut dyn PlayerSystem> = vec![
            &mut self.activity_system,
            &mut self.hero_system,
            &mut self.backpack_system,
            &mut self.building_system,
            &mut self.tech_system,
            &mut self.equip_system,
            &mut self.mail_system,
            &mut self.chat_system,
            &mut self.mission_system,
            &mut self.skin_system,
            &mut self.shop_system,
            &mut self.vip_system,
        ];

        for system in systems {
            if column == system.column_name() {
                system.clear_dirty();
                return true;
            }
        }

        false
    }

    fn append_registered_function_bases(&self, function_base: &mut Vec<FunctionClientBase>) {
        push_function_base(function_base, self.activity_system.to_function_base_bytes());
        push_function_base(function_base, self.hero_system.to_function_base_bytes());
        push_function_base(function_base, self.backpack_system.to_function_base_bytes());
        push_function_base(function_base, self.building_system.to_function_base_bytes());
        push_function_base(function_base, self.tech_system.to_function_base_bytes());
        push_function_base(function_base, self.equip_system.to_function_base_bytes());
        push_function_base(function_base, self.mail_system.to_function_base_bytes());
        push_function_base(function_base, self.chat_system.to_function_base_bytes());
        push_function_base(function_base, self.mission_system.to_function_base_bytes());
        push_function_base(function_base, self.skin_system.to_function_base_bytes());
        push_function_base(function_base, self.shop_system.to_function_base_bytes());
        push_function_base(function_base, self.vip_system.to_function_base_bytes());
    }

    fn dispatch_registered_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<SystemCommandResult> {
        let (response_payload, events) = match route_home_command(cmd) {
            Some(HomeCommandTarget::Mission) => self
                .mission_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Building) => self
                .building_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Hero) => self
                .hero_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Backpack) => self
                .backpack_system
                .handle_command_with_events_for_role(self.role_id, cmd, payload, config),
            Some(HomeCommandTarget::Technology) => self
                .tech_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Equip) => self
                .equip_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Mail) => self
                .mail_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Chat) => self
                .chat_system
                .handle_command_for_role(self.role_id, cmd, payload, config)
                .map(|response_payload| (response_payload, vec![])),
            Some(HomeCommandTarget::Vip) => self
                .vip_system
                .handle_command_with_events(cmd, payload, config),
            Some(HomeCommandTarget::Shop) => {
                return self.dispatch_shop_command(cmd, payload, config);
            }
            Some(HomeCommandTarget::Activity) => self
                .activity_system
                .handle_command_with_events(cmd, payload, config),
            None => return Err(anyhow::anyhow!("Unknown cmd: {}", cmd)),
        }?;

        Ok(SystemCommandResult {
            response_payload,
            events,
        })
    }
}

impl PlayerActor {
    fn dispatch_shop_command(
        &mut self,
        cmd: u32,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<SystemCommandResult> {
        match cmd {
            7651 => self.dispatch_shop_buy(payload, config),
            _ => Err(anyhow::anyhow!("Unsupported shop cmd: {}", cmd)),
        }
    }

    fn dispatch_shop_buy(
        &mut self,
        payload: &[u8],
        config: &Arc<StaticConfig>,
    ) -> Result<SystemCommandResult> {
        self.shop_system.ensure_configured(&config.shop);
        let rq = ShopBuyRq::decode(payload)?;
        let plan = self
            .shop_system
            .plan_buy(rq.shop_id, rq.slot, rq.buy_count, &config.shop)?;

        self.validate_shop_prices(&plan.prices)?;
        let mut events = Vec::new();
        self.apply_shop_prices(&plan.prices, &mut events)?;
        self.apply_shop_rewards(&plan.rewards, &mut events)?;
        let item = self.shop_system.apply_buy(&plan)?;

        let response_payload = ShopBuyRs {
            shop_id: Some(plan.shop_id),
            item: Some(item),
        }
        .encode_to_vec();

        Ok(SystemCommandResult {
            response_payload,
            events,
        })
    }

    fn validate_shop_prices(&self, prices: &[AwardPb]) -> Result<()> {
        for price in prices {
            if price.count <= 0 {
                continue;
            }
            match price.r#type {
                LORD_RESOURCE_AWARD_TYPE | LEGACY_LORD_RESOURCE_AWARD_TYPE => {
                    let have = self.lord_resource_amount(price.id)?;
                    if have < price.count {
                        anyhow::bail!(
                            "not enough lord resource id={}: have={}, need={}",
                            price.id,
                            have,
                            price.count
                        );
                    }
                }
                ITEM_AWARD_TYPE => {
                    let have = self.backpack_system.get_item_count(price.id);
                    if have < price.count {
                        anyhow::bail!(
                            "not enough item id={}: have={}, need={}",
                            price.id,
                            have,
                            price.count
                        );
                    }
                }
                other => anyhow::bail!("unsupported shop price award type={}", other),
            }
        }
        Ok(())
    }

    fn apply_shop_prices(&mut self, prices: &[AwardPb], events: &mut Vec<GameEvent>) -> Result<()> {
        for price in prices {
            if price.count <= 0 {
                continue;
            }
            match price.r#type {
                LORD_RESOURCE_AWARD_TYPE | LEGACY_LORD_RESOURCE_AWARD_TYPE => {
                    self.try_consume_lord_resource(price.id, price.count)?;
                    if price.id == LORD_RESOURCE_DIAMOND {
                        events.push(GameEvent::DiamondConsume {
                            role_id: self.role_id,
                            amount: price.count,
                        });
                    } else if price.id == LORD_RESOURCE_GOLD {
                        events.push(GameEvent::GoldConsume {
                            role_id: self.role_id,
                            amount: price.count,
                        });
                    } else {
                        events.push(GameEvent::ResourceChange {
                            resource_type: price.id,
                            delta: -price.count,
                        });
                    }
                }
                ITEM_AWARD_TYPE => {
                    let (ok, mut item_events) = self.backpack_system.consume_item_with_event(
                        self.role_id,
                        price.id,
                        price.count,
                    );
                    if !ok {
                        anyhow::bail!("not enough item id={}: need={}", price.id, price.count);
                    }
                    events.append(&mut item_events);
                }
                other => anyhow::bail!("unsupported shop price award type={}", other),
            }
        }
        Ok(())
    }

    fn apply_shop_rewards(
        &mut self,
        rewards: &[AwardPb],
        events: &mut Vec<GameEvent>,
    ) -> Result<()> {
        for reward in rewards {
            if reward.count <= 0 {
                continue;
            }
            match reward.r#type {
                LORD_RESOURCE_AWARD_TYPE | LEGACY_LORD_RESOURCE_AWARD_TYPE => {
                    self.grant_lord_resource(reward.id, reward.count)?;
                    events.push(GameEvent::ResourceChange {
                        resource_type: reward.id,
                        delta: reward.count,
                    });
                }
                ITEM_AWARD_TYPE => {
                    self.backpack_system.add_award(reward.clone());
                    events.push(GameEvent::ItemGain {
                        role_id: self.role_id,
                        prop_id: reward.id,
                        count: reward.count,
                    });
                }
                other => anyhow::bail!("unsupported shop reward award type={}", other),
            }
        }
        Ok(())
    }
}

fn push_function_base(function_base: &mut Vec<FunctionClientBase>, bytes: Vec<u8>) {
    if let Ok(f_base) = FunctionClientBase::decode(bytes.as_slice()) {
        function_base.push(f_base);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_existing_home_command_ranges() {
        assert_eq!(route_home_command(1101), Some(HomeCommandTarget::Mission));
        assert_eq!(route_home_command(1179), Some(HomeCommandTarget::Mission));
        assert_eq!(route_home_command(1501), Some(HomeCommandTarget::Building));
        assert_eq!(route_home_command(2001), Some(HomeCommandTarget::Hero));
        assert_eq!(route_home_command(4003), Some(HomeCommandTarget::Backpack));
        assert_eq!(
            route_home_command(4201),
            Some(HomeCommandTarget::Technology)
        );
        assert_eq!(route_home_command(4801), Some(HomeCommandTarget::Equip));
        assert_eq!(route_home_command(6001), Some(HomeCommandTarget::Mail));
        assert_eq!(route_home_command(7601), Some(HomeCommandTarget::Chat));
        assert_eq!(route_home_command(7603), Some(HomeCommandTarget::Chat));
        assert_eq!(route_home_command(7650), Some(HomeCommandTarget::Chat));
        assert_eq!(route_home_command(7401), Some(HomeCommandTarget::Vip));
        assert_eq!(route_home_command(7500), Some(HomeCommandTarget::Vip));
        assert_eq!(route_home_command(7651), Some(HomeCommandTarget::Shop));
        assert_eq!(route_home_command(7700), Some(HomeCommandTarget::Shop));
        assert_eq!(route_home_command(8001), Some(HomeCommandTarget::Activity));
        assert_eq!(route_home_command(10000), Some(HomeCommandTarget::Activity));
        assert_eq!(route_home_command(10001), None);
    }
}
