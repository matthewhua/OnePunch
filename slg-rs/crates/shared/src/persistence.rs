//! 玩家数据持久化层
//!
//! 对应 Java 版的 `PlayerDao` + `FunctionEntity` 存取体系。
//! 表结构完全对齐 `imperial_sim_game_hqy` 数据库。
//!
//! # 数据表结构
//!
//! - `p_account`：账号表（key_id 自增主键，account_key_id 是账号唯一ID）
//! - `p_lord`：领主基础数据（diamond/gold/meat/stamina 等核心资源）
//! - `p_data`：玩家功能模块数据（宽列，每个模块一个 blob 列）
//! - `p_global`：全服共享数据（按 server_id 分区，多个具名 blob 列）
//! - `p_server_config`：服务器配置参数
//!
//! # 存盘策略
//!
//! - 定时存盘：每 5 分钟，仅存 dirty 模块
//! - 下线存盘：全量存盘，完成后释放 Actor
//! - 紧急存盘：Actor panic 后序列化到本地文件兜底

use sqlx::MySqlPool;
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use chrono::Utc;

// ─── p_account 账号表 ─────────────────────────────────────────────────────────

/// 账号行（对应 p_account 表）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    pub key_id: i32,
    pub account_key_id: i64,
    pub server_id: i32,
    pub plat_no: i32,
    pub publisher: i32,
    pub plat_id: String,
    pub child_no: i32,
    pub forbid: i32,
    pub white_name: i32,
    pub role_id: i64,
    pub created: i32,
    pub device_no: Option<String>,
    pub create_date: Option<chrono::NaiveDateTime>,
    pub login_days: i32,
    pub login_date: Option<chrono::NaiveDateTime>,
    pub is_gm: i32,
    pub is_guider: i32,
    pub guidance: Option<i32>,
    pub log_off: Option<i8>,
    pub silence_time: Option<i64>,
    pub open_id: Option<String>,
    pub pack_id: Option<String>,
}

// ─── p_lord 领主基础数据 ──────────────────────────────────────────────────────

/// 领主基础数据（对应 p_lord 表）
///
/// 存储领主的核心资源和状态，不走 blob 序列化，直接按列读写。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LordRow {
    pub role_id: i64,
    pub nick: Option<String>,
    pub portrait: Option<String>,
    pub portrait_frame: Option<i32>,
    pub top_up: Option<String>,  // DECIMAL(11,2) 存为字符串避免额外依赖
    pub diamond: Option<i64>,
    pub diamond_cost: Option<i64>,
    pub guide_id: Option<i32>,
    pub on_time: Option<i32>,
    pub ol_time: Option<i32>,
    pub off_time: Option<i32>,
    pub ol_month: Option<i32>,
    pub title: Option<i32>,
    pub max_key: Option<i32>,
    pub role_status: Option<String>,
    pub across_day_deal_time: Option<i32>,
    pub battle_fight: Option<i64>,
    pub meat: Option<i64>,
    pub fame: Option<i32>,
    pub gold: Option<i64>,
    pub search_survivor_time: Option<i32>,
    pub stamina: Option<i64>,
    pub start_ad_time: Option<i32>,
    pub start_ad_id: Option<i32>,
    pub is_add_login: Option<i32>,
    pub total_login: Option<i32>,
    pub current_streak: Option<i32>,
    pub vip_level: Option<i32>,
    pub vip_exp: Option<i32>,
    pub camp_id: Option<i32>,
    pub last_periodic_task_time: Option<String>,
    pub lord_system_setting: Option<String>,
    pub pay_amount: Option<i32>,
    pub language: Option<String>,
    pub push_switch: Option<String>,
}

// ─── p_data 功能模块数据（宽列）─────────────────────────────────────────────

/// 玩家功能模块数据（对应 p_data 表，宽列设计）
///
/// 每个功能模块对应一个 blob 列，存储 protobuf 序列化的数据。
/// 列名与 Java 版 FunctionEntity 的字段名完全一致。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerDataRow {
    pub role_id: i64,
    pub hero_func: Option<Vec<u8>>,
    pub sim_func: Option<Vec<u8>>,
    pub backpack_func: Option<Vec<u8>>,
    pub technology_func: Option<Vec<u8>>,
    pub combat_func: Option<Vec<u8>>,
    pub equip_func: Option<Vec<u8>>,
    pub world_func: Option<Vec<u8>>,
    pub pay_func: Option<Vec<u8>>,
    pub mail_func: Option<Vec<u8>>,
    pub guise_func: Option<Vec<u8>>,
    pub intel_broker_func: Option<Vec<u8>>,
    pub camp_func: Option<Vec<u8>>,
    pub activity_func: Option<Vec<u8>>,
    pub vip_func: Option<Vec<u8>>,
    pub wall_func: Option<Vec<u8>>,
    pub shop_func: Option<Vec<u8>>,
    pub lord_talent_func: Option<Vec<u8>>,
    pub mission_func: Option<Vec<u8>>,
    pub game_play_func: Option<Vec<u8>>,
    pub arena_func: Option<Vec<u8>>,
    pub lord_equip_func: Option<Vec<u8>>,
    pub skin_func: Option<Vec<u8>>,
    pub chat_func: Option<Vec<u8>>,
    pub social_func: Option<Vec<u8>>,
    pub milestone_func: Option<Vec<u8>>,
}

/// 待存盘的单个模块数据
#[derive(Debug)]
pub struct SaveEntry {
    /// p_data 表的列名（如 "activity_func"）
    pub column: &'static str,
    pub data: Vec<u8>,
}

// ─── p_global 全服共享数据 ────────────────────────────────────────────────────

/// 全服共享数据（对应 p_global 表）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GlobalDataRow {
    pub server_id: i32,
    pub common_mail: Option<Vec<u8>>,
    pub map_data: Option<Vec<u8>>,
    pub camp_data: Option<Vec<u8>>,
    pub activity_global: Option<Vec<u8>>,
    pub rank_data: Option<Vec<u8>>,
    pub chat_data: Option<Vec<u8>>,
    pub gameplay_global_data: Option<Vec<u8>>,
    pub milestone_global_data: Option<Vec<u8>>,
}

// ─── p_server_config 服务器配置 ───────────────────────────────────────────────

/// 服务器配置行（对应 p_server_config 表）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerConfigRow {
    pub param_id: i32,
    pub title: String,
    pub param_name: String,
    pub param_value: String,
    pub descs: String,
}

// ─── PlayerDao ────────────────────────────────────────────────────────────────

/// 玩家数据访问对象
///
/// 封装所有 p_account / p_lord / p_data 的数据库操作。
/// 通过 `Arc<PlayerDao>` 在 PlayerManager 和 PlayerActor 间共享。
#[derive(Clone)]
pub struct PlayerDao {
    pool: MySqlPool,
}

impl PlayerDao {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // ── p_account 操作 ──

    /// 根据 account_key_id + server_id 查询账号
    pub async fn get_account(&self, account_key_id: i64, server_id: i32) -> Result<Option<AccountRow>> {
        sqlx::query_as::<_, AccountRow>(
            "SELECT * FROM p_account WHERE account_key_id = ? AND server_id = ?"
        )
        .bind(account_key_id)
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query p_account")
    }

    /// 根据 role_id 查询账号
    pub async fn get_account_by_role(&self, role_id: i64) -> Result<Option<AccountRow>> {
        sqlx::query_as::<_, AccountRow>(
            "SELECT * FROM p_account WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query p_account by role_id")
    }

    /// 创建账号记录（首次登录）
    pub async fn create_account(
        &self,
        account_key_id: i64,
        server_id: i32,
        plat_no: i32,
        plat_id: &str,
        role_id: i64,
        device_no: Option<&str>,
        pack_id: Option<&str>,
    ) -> Result<i32> {
        let result = sqlx::query(
            "INSERT INTO p_account \
             (account_key_id, server_id, plat_no, plat_id, role_id, device_no, pack_id, \
              created, login_days, create_date, login_date) \
             VALUES (?, ?, ?, ?, ?, ?, ?, 1, 1, NOW(), NOW())"
        )
        .bind(account_key_id)
        .bind(server_id)
        .bind(plat_no)
        .bind(plat_id)
        .bind(role_id)
        .bind(device_no)
        .bind(pack_id)
        .execute(&self.pool)
        .await
        .context("Failed to insert p_account")?;

        let key_id = result.last_insert_id() as i32;
        info!(key_id, account_key_id, server_id, role_id, "Account created");
        Ok(key_id)
    }

    /// 更新登录信息（login_date、login_days）
    pub async fn update_login(&self, role_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE p_account SET login_date = NOW(), \
             login_days = login_days + 1 \
             WHERE role_id = ?"
        )
        .bind(role_id)
        .execute(&self.pool)
        .await
        .context("Failed to update p_account login")?;
        Ok(())
    }

    /// 更新 log_off 标记（下线时置 1，上线时置 0）
    pub async fn set_log_off(&self, role_id: i64, log_off: bool) -> Result<()> {
        sqlx::query("UPDATE p_account SET log_off = ? WHERE role_id = ?")
            .bind(log_off as i8)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .context("Failed to update log_off")?;
        Ok(())
    }

    // ── p_lord 操作 ──

    /// 加载领主基础数据
    pub async fn load_lord(&self, role_id: i64) -> Result<Option<LordRow>> {
        sqlx::query_as::<_, LordRow>(
            "SELECT * FROM p_lord WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load p_lord")
    }

    /// 创建领主基础数据（新玩家）
    pub async fn create_lord(
        &self,
        role_id: i64,
        nick: &str,
        camp_id: i32,
        on_time: i32,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO p_lord \
             (role_id, nick, diamond, gold, meat, stamina, vip_level, vip_exp, \
              camp_id, on_time, ol_time, off_time, total_login, current_streak, \
              battle_fight, fame, pay_amount) \
             VALUES (?, ?, 0, 0, 0, 200, 0, 0, ?, ?, 0, 0, 1, 1, 0, 0, 0)"
        )
        .bind(role_id)
        .bind(nick)
        .bind(camp_id)
        .bind(on_time)
        .execute(&self.pool)
        .await
        .context("Failed to insert p_lord")?;
        info!(role_id, nick, camp_id, "Lord created");
        Ok(())
    }

    /// 存盘领主基础数据（全量更新）
    pub async fn save_lord(&self, lord: &LordRow) -> Result<()> {
        sqlx::query(
            "UPDATE p_lord SET \
             nick = ?, portrait = ?, portrait_frame = ?, \
             diamond = ?, diamond_cost = ?, top_up = ?, \
             guide_id = ?, ol_time = ?, off_time = ?, ol_month = ?, \
             title = ?, max_key = ?, role_status = ?, \
             across_day_deal_time = ?, battle_fight = ?, \
             meat = ?, fame = ?, gold = ?, stamina = ?, \
             is_add_login = ?, total_login = ?, current_streak = ?, \
             vip_level = ?, vip_exp = ?, camp_id = ?, \
             last_periodic_task_time = ?, lord_system_setting = ?, \
             pay_amount = ?, language = ?, push_switch = ? \
             WHERE role_id = ?"
        )
        .bind(&lord.nick)
        .bind(&lord.portrait)
        .bind(lord.portrait_frame)
        .bind(lord.diamond)
        .bind(lord.diamond_cost)
        .bind(lord.top_up.clone())
        .bind(lord.guide_id)
        .bind(lord.ol_time)
        .bind(lord.off_time)
        .bind(lord.ol_month)
        .bind(lord.title)
        .bind(lord.max_key)
        .bind(&lord.role_status)
        .bind(lord.across_day_deal_time)
        .bind(lord.battle_fight)
        .bind(lord.meat)
        .bind(lord.fame)
        .bind(lord.gold)
        .bind(lord.stamina)
        .bind(lord.is_add_login)
        .bind(lord.total_login)
        .bind(lord.current_streak)
        .bind(lord.vip_level)
        .bind(lord.vip_exp)
        .bind(lord.camp_id)
        .bind(&lord.last_periodic_task_time)
        .bind(&lord.lord_system_setting)
        .bind(lord.pay_amount)
        .bind(&lord.language)
        .bind(&lord.push_switch)
        .bind(lord.role_id)
        .execute(&self.pool)
        .await
        .context("Failed to save p_lord")?;
        Ok(())
    }

    /// 更新领主在线时长（下线时调用）
    pub async fn update_lord_offline(&self, role_id: i64, ol_time_delta: i32, off_time: i32) -> Result<()> {
        sqlx::query(
            "UPDATE p_lord SET ol_time = ol_time + ?, off_time = ? WHERE role_id = ?"
        )
        .bind(ol_time_delta)
        .bind(off_time)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .context("Failed to update lord offline")?;
        Ok(())
    }

    // ── p_data 操作（宽列）──

    /// 加载玩家所有功能模块数据（单次查询全部列）
    pub async fn load_player_data(&self, role_id: i64) -> Result<Option<PlayerDataRow>> {
        sqlx::query_as::<_, PlayerDataRow>(
            "SELECT * FROM p_data WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load p_data")
    }

    /// 初始化新玩家的 p_data 行（全列 NULL）
    pub async fn init_player_data(&self, role_id: i64) -> Result<()> {
        sqlx::query("INSERT IGNORE INTO p_data (role_id) VALUES (?)")
            .bind(role_id)
            .execute(&self.pool)
            .await
            .context("Failed to init p_data")?;
        Ok(())
    }

    /// 批量存盘功能模块数据（事务，动态构建 UPDATE SET 子句）
    ///
    /// 只更新 dirty 的列，避免全量写入。
    pub async fn save_player_data(&self, role_id: i64, entries: &[SaveEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        // 动态构建 SET 子句：col1 = ?, col2 = ?, ...
        let set_clause: String = entries
            .iter()
            .map(|e| format!("{} = ?", e.column))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!("UPDATE p_data SET {} WHERE role_id = ?", set_clause);

        let mut query = sqlx::query(&sql);
        for entry in entries {
            query = query.bind(&entry.data);
        }
        query = query.bind(role_id);

        query.execute(&self.pool)
            .await
            .context("Failed to save p_data")?;

        Ok(())
    }

    // ── p_global 操作 ──

    /// 加载全服共享数据
    pub async fn load_global(&self, server_id: i32) -> Result<Option<GlobalDataRow>> {
        sqlx::query_as::<_, GlobalDataRow>(
            "SELECT * FROM p_global WHERE server_id = ?"
        )
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load p_global")
    }

    /// 初始化全服数据行
    pub async fn init_global(&self, server_id: i32) -> Result<()> {
        sqlx::query("INSERT IGNORE INTO p_global (server_id) VALUES (?)")
            .bind(server_id)
            .execute(&self.pool)
            .await
            .context("Failed to init p_global")?;
        Ok(())
    }

    /// 更新全服共享数据的指定列
    pub async fn save_global_column(&self, server_id: i32, column: &str, data: &[u8]) -> Result<()> {
        // 列名来自内部常量，不存在 SQL 注入风险
        let sql = format!("UPDATE p_global SET {} = ? WHERE server_id = ?", column);
        sqlx::query(&sql)
            .bind(data)
            .bind(server_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to save p_global column={}", column))?;
        Ok(())
    }

    // ── p_server_config 操作 ──

    /// 加载所有服务器配置
    pub async fn load_server_config(&self) -> Result<Vec<ServerConfigRow>> {
        sqlx::query_as::<_, ServerConfigRow>(
            "SELECT * FROM p_server_config"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load p_server_config")
    }

    /// 根据 param_name 查询配置值
    pub async fn get_config_value(&self, param_name: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT param_value FROM p_server_config WHERE param_name = ?"
        )
        .bind(param_name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query p_server_config")?;
        Ok(row.map(|r| r.0))
    }
}

// ─── 功能模块列名常量 ─────────────────────────────────────────────────────────
//
// 对应 p_data 表的列名，与 Java 版 FunctionEntity 字段名完全一致。
// PlayerSystem::column_name() 返回这些常量。

/// p_data 表功能模块列名定义
pub mod col {
    pub const HERO: &str = "hero_func";
    pub const SIM: &str = "sim_func";
    pub const BACKPACK: &str = "backpack_func";
    pub const TECHNOLOGY: &str = "technology_func";
    pub const COMBAT: &str = "combat_func";
    pub const EQUIP: &str = "equip_func";
    pub const WORLD: &str = "world_func";
    pub const PAY: &str = "pay_func";
    pub const MAIL: &str = "mail_func";
    pub const GUISE: &str = "guise_func";
    pub const INTEL_BROKER: &str = "intel_broker_func";
    pub const CAMP: &str = "camp_func";
    pub const ACTIVITY: &str = "activity_func";
    pub const VIP: &str = "vip_func";
    pub const WALL: &str = "wall_func";
    pub const SHOP: &str = "shop_func";
    pub const LORD_TALENT: &str = "lord_talent_func";
    pub const MISSION: &str = "mission_func";
    pub const GAME_PLAY: &str = "game_play_func";
    pub const ARENA: &str = "arena_func";
    pub const LORD_EQUIP: &str = "lord_equip_func";
    pub const SKIN: &str = "skin_func";
    pub const CHAT: &str = "chat_func";
    pub const SOCIAL: &str = "social_func";
    pub const MILESTONE: &str = "milestone_func";
}

/// p_global 表列名定义
pub mod global_col {
    pub const COMMON_MAIL: &str = "common_mail";
    pub const MAP_DATA: &str = "map_data";
    pub const CAMP_DATA: &str = "camp_data";
    pub const ACTIVITY_GLOBAL: &str = "activity_global";
    pub const RANK_DATA: &str = "rank_data";
    pub const CHAT_DATA: &str = "chat_data";
    pub const GAMEPLAY_GLOBAL: &str = "gameplay_global_data";
    pub const MILESTONE_GLOBAL: &str = "milestone_global_data";
}

// ─── 紧急存盘 ─────────────────────────────────────────────────────────────────

/// 紧急存盘：将数据序列化到本地文件
///
/// 当 Actor panic 或数据库不可用时，作为兜底方案。
/// 文件路径：`{emergency_dir}/role_{role_id}_{timestamp}.bin`
pub fn emergency_save_to_file(
    emergency_dir: &str,
    role_id: i64,
    entries: &[SaveEntry],
) {
    use std::fs;
    use std::io::Write;

    if let Err(e) = fs::create_dir_all(emergency_dir) {
        error!(role_id, "Failed to create emergency save dir: {}", e);
        return;
    }

    let timestamp = Utc::now().timestamp();
    let path = format!("{}/role_{}_{}.bin", emergency_dir, role_id, timestamp);

    let mut file = match fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            error!(role_id, path, "Failed to create emergency save file: {}", e);
            return;
        }
    };

    // 格式：[entry_count: u32] [col_len: u8, col: bytes, data_len: u32, data: bytes]...
    let count = entries.len() as u32;
    if file.write_all(&count.to_le_bytes()).is_err() { return; }

    for entry in entries {
        let col_bytes = entry.column.as_bytes();
        let _ = file.write_all(&[col_bytes.len() as u8]);
        let _ = file.write_all(col_bytes);
        let len = entry.data.len() as u32;
        let _ = file.write_all(&len.to_le_bytes());
        let _ = file.write_all(&entry.data);
    }

    warn!(role_id, path, entries = entries.len(), "Emergency save completed");
}
