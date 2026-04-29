//! 玩家数据持久化层
//!
//! 对应 Java 版的 `PlayerDao` + `FunctionEntity` 存取体系。
//!
//! # 数据表结构
//!
//! - `p_role`：玩家角色基础信息（account_id, nickname, level, camp 等）
//! - `p_data`：玩家功能模块数据（role_id + keyId → blob），KV 结构
//! - `p_global`：全服共享数据（key → blob）
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

// ─── p_role 角色基础信息 ──────────────────────────────────────────────────────

/// 角色基础信息（对应 p_role 表）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoleRow {
    pub role_id: i64,
    pub account_id: i64,
    pub nickname: String,
    pub level: i32,
    #[sqlx(rename = "serverId")]
    pub server_id: i32,
    pub camp: i32,
    pub create_time: Option<chrono::NaiveDateTime>,
    pub login_time: Option<chrono::NaiveDateTime>,
    pub logout_time: Option<chrono::NaiveDateTime>,
}

// ─── p_data 功能模块数据 ──────────────────────────────────────────────────────

/// 功能模块数据行（对应 p_data 表的一行）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataRow {
    pub role_id: i64,
    #[sqlx(rename = "keyId")]
    pub key_id: i32,
    pub data: Option<Vec<u8>>,
    pub update_time: Option<chrono::NaiveDateTime>,
}

/// 待存盘的单个模块数据
#[derive(Debug)]
pub struct SaveEntry {
    pub key_id: i32,
    pub data: Vec<u8>,
}

// ─── PlayerDao ────────────────────────────────────────────────────────────────

/// 玩家数据访问对象
///
/// 封装所有 p_role / p_data 的数据库操作。
/// 通过 `Arc<PlayerDao>` 在 PlayerManager 和 PlayerActor 间共享。
#[derive(Clone)]
pub struct PlayerDao {
    pool: MySqlPool,
}

impl PlayerDao {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // ── p_role 操作 ──

    /// 根据 account_id + server_id 查询角色
    pub async fn get_role_by_account(&self, account_id: i64, server_id: i32) -> Result<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT * FROM p_role WHERE account_id = ? AND serverId = ?"
        )
        .bind(account_id)
        .bind(server_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query p_role")?;
        Ok(row)
    }

    /// 根据 role_id 查询角色
    pub async fn get_role(&self, role_id: i64) -> Result<Option<RoleRow>> {
        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT * FROM p_role WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query p_role by role_id")?;
        Ok(row)
    }

    /// 创建新角色
    pub async fn create_role(
        &self,
        account_id: i64,
        server_id: i32,
        nickname: &str,
        camp: i32,
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO p_role (account_id, serverId, nickname, camp, create_time) VALUES (?, ?, ?, ?, NOW())"
        )
        .bind(account_id)
        .bind(server_id)
        .bind(nickname)
        .bind(camp)
        .execute(&self.pool)
        .await
        .context("Failed to insert p_role")?;

        let role_id = result.last_insert_id() as i64;
        info!(role_id, account_id, server_id, nickname, "New role created");
        Ok(role_id)
    }

    /// 更新登录时间
    pub async fn update_login_time(&self, role_id: i64) -> Result<()> {
        sqlx::query("UPDATE p_role SET login_time = NOW() WHERE role_id = ?")
            .bind(role_id)
            .execute(&self.pool)
            .await
            .context("Failed to update login_time")?;
        Ok(())
    }

    /// 更新登出时间
    pub async fn update_logout_time(&self, role_id: i64) -> Result<()> {
        sqlx::query("UPDATE p_role SET logout_time = NOW() WHERE role_id = ?")
            .bind(role_id)
            .execute(&self.pool)
            .await
            .context("Failed to update logout_time")?;
        Ok(())
    }

    /// 更新角色等级
    pub async fn update_level(&self, role_id: i64, level: i32) -> Result<()> {
        sqlx::query("UPDATE p_role SET level = ? WHERE role_id = ?")
            .bind(level)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .context("Failed to update level")?;
        Ok(())
    }

    // ── p_data 操作 ──

    /// 加载玩家所有功能模块数据
    pub async fn load_all_data(&self, role_id: i64) -> Result<Vec<DataRow>> {
        let rows = sqlx::query_as::<_, DataRow>(
            "SELECT * FROM p_data WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to load p_data")?;
        Ok(rows)
    }

    /// 加载单个功能模块数据
    pub async fn load_data(&self, role_id: i64, key_id: i32) -> Result<Option<Vec<u8>>> {
        let row: Option<(Option<Vec<u8>>,)> = sqlx::query_as(
            "SELECT data FROM p_data WHERE role_id = ? AND keyId = ?"
        )
        .bind(role_id)
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load p_data entry")?;
        Ok(row.and_then(|r| r.0))
    }

    /// 批量存盘（事务写入，仅写 dirty 模块）
    ///
    /// 使用 INSERT ... ON DUPLICATE KEY UPDATE 实现 upsert。
    /// 所有 dirty 模块在同一个事务中写入，保证原子性。
    pub async fn save_data(&self, role_id: i64, entries: &[SaveEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await
            .context("Failed to begin transaction")?;

        for entry in entries {
            sqlx::query(
                "INSERT INTO p_data (role_id, keyId, data, update_time) VALUES (?, ?, ?, NOW()) \
                 ON DUPLICATE KEY UPDATE data = VALUES(data), update_time = NOW()"
            )
            .bind(role_id)
            .bind(entry.key_id)
            .bind(&entry.data)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to save p_data keyId={}", entry.key_id))?;
        }

        tx.commit().await
            .context("Failed to commit save transaction")?;

        Ok(())
    }

    /// 初始化新玩家的功能模块数据（全部 key 插入空 blob）
    pub async fn init_player_data(&self, role_id: i64, key_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await
            .context("Failed to begin init transaction")?;

        for &key_id in key_ids {
            sqlx::query(
                "INSERT IGNORE INTO p_data (role_id, keyId, data) VALUES (?, ?, NULL)"
            )
            .bind(role_id)
            .bind(key_id)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to init p_data keyId={}", key_id))?;
        }

        tx.commit().await
            .context("Failed to commit init transaction")?;

        Ok(())
    }

    // ── p_global 操作 ──

    /// 加载全局数据
    pub async fn load_global(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let row: Option<(Option<Vec<u8>>,)> = sqlx::query_as(
            "SELECT data FROM p_global WHERE `key` = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load p_global")?;
        Ok(row.and_then(|r| r.0))
    }

    /// 保存全局数据
    pub async fn save_global(&self, key: &str, data: &[u8]) -> Result<()> {
        sqlx::query(
            "INSERT INTO p_global (`key`, data, update_time) VALUES (?, ?, NOW()) \
             ON DUPLICATE KEY UPDATE data = VALUES(data), update_time = NOW()"
        )
        .bind(key)
        .bind(data)
        .execute(&self.pool)
        .await
        .context("Failed to save p_global")?;
        Ok(())
    }
}

// ─── 功能模块 keyId 常量 ─────────────────────────────────────────────────────
//
// 对应 Java 版 FunctionTypeDefine，与 p_data.keyId 一一对应。
// 注意：这些值必须与 Java 版保持一致，否则数据不兼容。

/// 功能模块 keyId 定义（对应 p_data.keyId）
pub mod key_id {
    pub const LORD: i32 = 0;
    pub const BUILDING: i32 = 1;
    pub const HERO: i32 = 2;
    pub const BAG: i32 = 3;
    pub const TECH: i32 = 4;
    pub const EQUIP: i32 = 5;
    pub const MISSION: i32 = 6;
    pub const COMBAT: i32 = 7;
    pub const WORLD: i32 = 8;
    pub const PAY: i32 = 9;
    pub const MAIL: i32 = 10;
    pub const GUISE: i32 = 11;
    pub const INTEL_BROKER: i32 = 12;
    pub const VIP: i32 = 13;
    pub const ACTIVITY: i32 = 14;
    pub const CAMP: i32 = 15;
    pub const WALL: i32 = 16;
    pub const GAMEPLAY: i32 = 17;
    pub const CHAT: i32 = 18;
    pub const SHOP: i32 = 19;
    pub const LORD_TALENT: i32 = 20;
    pub const ARENA: i32 = 21;
    pub const SKIN: i32 = 22;
    pub const LORD_EQUIP: i32 = 23;
    pub const SOCIAL: i32 = 24;

    /// 所有已知的 keyId 列表（用于新玩家初始化）
    pub const ALL: &[i32] = &[
        LORD, BUILDING, HERO, BAG, TECH, EQUIP, MISSION, COMBAT, WORLD,
        PAY, MAIL, GUISE, INTEL_BROKER, VIP, ACTIVITY, CAMP, WALL,
        GAMEPLAY, CHAT, SHOP, LORD_TALENT, ARENA, SKIN, LORD_EQUIP, SOCIAL,
    ];
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

    // 简单格式：[entry_count: u32] [key_id: i32, data_len: u32, data: bytes]...
    let count = entries.len() as u32;
    if file.write_all(&count.to_le_bytes()).is_err() { return; }

    for entry in entries {
        let _ = file.write_all(&entry.key_id.to_le_bytes());
        let len = entry.data.len() as u32;
        let _ = file.write_all(&len.to_le_bytes());
        let _ = file.write_all(&entry.data);
    }

    warn!(role_id, path, entries = entries.len(), "Emergency save completed");
}
