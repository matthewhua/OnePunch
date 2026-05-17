use anyhow::Result;
use prost::Message;
use proto::slg::FunctionClientBase;
use shared::event::GameEvent;
use shared::msg::ToFunctionClientBaseBytes;
use shared::persistence::{PlayerDataRow, SaveEntry};
use shared::static_config::StaticConfig;
use std::sync::Arc;
use tracing::{error, warn};

use crate::actors::player_actor::PlayerActor;
use crate::systems::PlayerSystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeCommandTarget {
    Mission,
    Building,
    Hero,
    Backpack,
    Technology,
    Equip,
    Mail,
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
            (&mut self.mission_system, row.mission_func.as_ref()),
            (&mut self.skin_system, row.skin_func.as_ref()),
        ];

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
            &mut self.mission_system,
            &mut self.skin_system,
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
            &mut self.mission_system,
            &mut self.skin_system,
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
            &mut self.mission_system,
            &mut self.skin_system,
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
            &mut self.mission_system,
            &mut self.skin_system,
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
        push_function_base(function_base, self.mission_system.to_function_base_bytes());
        push_function_base(function_base, self.skin_system.to_function_base_bytes());
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
        assert_eq!(route_home_command(8001), Some(HomeCommandTarget::Activity));
        assert_eq!(route_home_command(10000), Some(HomeCommandTarget::Activity));
        assert_eq!(route_home_command(10001), None);
    }
}
