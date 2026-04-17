/// 游戏指令集枚举
/// 对应客户端通讯中的 Cmd ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GameCmd {
    // 账号相关 (1000-1100)
    LoginRq = 1001,
    LoginRs = 1002,
    
    // 游戏通用 (1100-1200)
    BeginGameRq = 1101,
    BeginGameRs = 1102,
    RoleLoginRq = 1107,
    RoleLoginRs = 1108,
    
    // 心跳与系统 (9000-9999)
    Heartbeat = 9001,
    
    // 未知
    Unknown = 0,
}

impl From<u32> for GameCmd {
    fn from(id: u32) -> Self {
        match id {
            1001 => GameCmd::LoginRq,
            1002 => GameCmd::LoginRs,
            1101 => GameCmd::BeginGameRq,
            1102 => GameCmd::BeginGameRs,
            1107 => GameCmd::RoleLoginRq,
            1108 => GameCmd::RoleLoginRs,
            9001 => GameCmd::Heartbeat,
            _ => GameCmd::Unknown,
        }
    }
}

impl Into<u32> for GameCmd {
    fn into(self) -> u32 {
        self as u32
    }
}
