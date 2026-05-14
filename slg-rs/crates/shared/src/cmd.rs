// GameCmd 枚举和 From<u32>/From<GameCmd> 实现由 proto crate 的 build.rs 自动生成
// 来源：proto 文件中 `extend Base { optional XxxMsg ext = N; }` 定义
pub use proto::cmd::GameCmd;

/// 指令路由目标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdRoute {
    /// 认证相关（Gateway 直接处理或转发 Auth Service）
    Auth,
    /// 玩家个人数据（Home Service）
    Home,
    /// 世界地图 / 行军 / 战斗（World Service）
    World,
}

/// GameCmd 扩展方法（路由 + 推送判断）
///
/// 由于 GameCmd 定义在 proto crate，Rust 的 orphan rule 不允许在 shared crate
/// 中直接 `impl GameCmd`，因此通过 extension trait 提供额外方法。
pub trait GameCmdExt {
    /// 根据命令号判断路由目标
    fn route(self) -> CmdRoute;
    /// 是否是服务端主动推送消息
    fn is_push(self) -> bool;
}

impl GameCmdExt for GameCmd {
    /// 根据命令号判断路由目标
    ///
    /// 路由规则：
    /// - 1001-1108：登录认证，Gateway 直接处理或转发 Auth Service
    /// - 50001-50020、50022-50032、50034-50040、5101-5104、5121-5124、5141-5148、
    ///   5161-5166、5201-5202、5221-5224：World.proto 顶层世界地图/行军/战斗命令
    /// - 其余：玩家个人数据，转发 Home Service
    ///
    /// 注意：50021 和 50033 是顶层命令表空洞，它们是 DispatchTroopRq 下的嵌套动作扩展，
    /// 不能按顶层 World 命令转发。
    fn route(self) -> CmdRoute {
        let id = self as u32;
        match id {
            1001..=1108 => CmdRoute::Auth,
            50001..=50020
            | 50022..=50032
            | 50034..=50040
            | 5101..=5104
            | 5121..=5124
            | 5141..=5148
            | 5161..=5166
            | 5201..=5202
            | 5221..=5224 => CmdRoute::World,
            _ => CmdRoute::Home,
        }
    }

    /// 是否是服务端主动推送消息（Sync 开头 + Rs 结尾，无对应 Rq）
    fn is_push(self) -> bool {
        let name = format!("{:?}", self);
        name.starts_with("Sync") && name.ends_with("Rs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_world_commands_to_world_service() {
        assert_eq!(GameCmd::from(50019).route(), CmdRoute::World);
        assert_eq!(GameCmd::from(50022).route(), CmdRoute::World);
        assert_eq!(GameCmd::from(50040).route(), CmdRoute::World);
        assert_eq!(GameCmd::from(5101).route(), CmdRoute::World);
        assert_eq!(GameCmd::from(5201).route(), CmdRoute::World);
    }

    #[test]
    fn keeps_gap_commands_outside_the_world_route() {
        assert_eq!(GameCmd::from(50021).route(), CmdRoute::Home);
        assert_eq!(GameCmd::from(50033).route(), CmdRoute::Home);
    }

    #[test]
    fn keeps_existing_home_and_auth_routes() {
        assert_eq!(GameCmd::from(1101).route(), CmdRoute::Auth);
        assert_eq!(GameCmd::from(8001).route(), CmdRoute::Home);
    }
}
