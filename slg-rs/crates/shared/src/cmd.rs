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
    /// - 100..=106：登录认证，Gateway 直接处理或转发 Auth Service
    /// - 50000+：世界地图/行军/战斗，转发 World Service
    /// - 其余：玩家个人数据，转发 Home Service
    fn route(self) -> CmdRoute {
        let id = self as u32;
        match id {
            103..=106 => CmdRoute::Auth,
            50001..=59999 => CmdRoute::World,
            _ => CmdRoute::Home,
        }
    }

    /// 是否是服务端主动推送消息（Sync 开头 + Rs 结尾，无对应 Rq）
    fn is_push(self) -> bool {
        let name = format!("{:?}", self);
        name.starts_with("Sync") && name.ends_with("Rs")
    }
}
