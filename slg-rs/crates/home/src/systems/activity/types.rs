/// 活动玩法类型（排除战令和社区跳转，这两种留在 Java 侧处理）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u32)]
pub enum ActivityFormType {
    Sign = 1,
    Task = 2,
    ScoreAward = 3,
    Shop = 4,
    Giftpack = 5,
    OptPack = 6,
    RechargeAward = 7,
    Rank = 8,
    Turntable = 9,
    // Battlepass = 10,      // 不迁移
    Questionnaire = 11,
    TaskGroup = 12,
    SupremeLord = 13,
    Voyage = 14,
    Monopoly = 15,
    Bank = 16,
    HeroHall = 17,
    Milestone = 18,
    MilestoneBoss = 19,
    // CommunityJump = 20,   // 不迁移
}

/// 活动生命周期阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivityStage {
    /// 预显期
    PreDisplay,
    /// 开启期
    Open,
    /// 结束展示期
    EndDisplay,
    /// 关闭期
    Closed,
}
