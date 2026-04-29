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

impl GameCmd {
    /// 根据命令号判断路由目标
    ///
    /// 路由规则：
    /// - 1001-1108：登录认证，Gateway 直接处理或转发 Auth Service
    /// - 2000-3999：世界地图/行军/战斗，转发 World Service
    /// - 其余：玩家个人数据，转发 Home Service
    pub fn route(self) -> CmdRoute {
        let id = self as u32;
        match id {
            1001..=1108 => CmdRoute::Auth,
            2000..=3999 => CmdRoute::World,
            _ => CmdRoute::Home,
        }
    }

    /// 是否是服务端主动推送消息（Sync 开头 + Rs 结尾，无对应 Rq）
    pub fn is_push(self) -> bool {
        let name = format!("{:?}", self);
        name.starts_with("Sync") && name.ends_with("Rs")
    }
}
