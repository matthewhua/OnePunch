//! 静态配置系统
//!
//! 服务启动时从 MySQL `s_` 前缀表批量加载所有策划配置到内存，
//! 运行时通过 `Arc<StaticConfig>` 只读访问。
//! 支持 GM 命令触发热加载，通过 `watch::channel` 广播给所有 Actor。
//!
//! # 使用方式
//!
//! ```ignore
//! // 启动时
//! let (watcher, config_rx) = ConfigWatcher::new(pool).await?;
//! let config = config_rx.borrow().clone(); // Arc<StaticConfig>
//!
//! // Actor 中监听热加载
//! tokio::select! {
//!     _ = config_rx.changed() => {
//!         let new_config = config_rx.borrow().clone();
//!     }
//! }
//!
//! // GM 触发热加载
//! watcher.reload().await?;
//! ```

pub mod activity;
pub mod hero;
pub mod building;
pub mod tech;
pub mod equip;
pub mod item;
pub mod task;
pub mod shop;
pub mod vip;
pub mod skin;
pub mod lord_equip;
pub mod lord_talent;
pub mod mail;
pub mod world;
pub mod battle;
pub mod pay;

pub use activity::ActivityConfig;
pub use hero::HeroConfig;
pub use building::BuildingConfig;
pub use tech::TechConfig;
pub use equip::EquipConfig;
pub use item::ItemConfig;
pub use task::TaskConfig;
pub use shop::ShopConfig;
pub use vip::VipConfig;
pub use skin::SkinConfig;
pub use lord_equip::LordEquipConfig;
pub use lord_talent::LordTalentConfig;
pub use mail::MailConfig;
pub use world::WorldConfig;
pub use battle::BattleConfig;
pub use pay::PayConfig;

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::watch;
use tracing::{info, warn};

/// 全部静态配置的聚合容器
///
/// 通过 `Arc<StaticConfig>` 在各 Actor 间共享，热加载时整体替换。
#[derive(Debug, Clone, Default)]
pub struct StaticConfig {
    pub activity: ActivityConfig,
    pub hero: HeroConfig,
    pub building: BuildingConfig,
    pub tech: TechConfig,
    pub equip: EquipConfig,
    pub item: ItemConfig,
    pub task: TaskConfig,
    pub shop: ShopConfig,
    pub vip: VipConfig,
    pub skin: SkinConfig,
    pub lord_equip: LordEquipConfig,
    pub lord_talent: LordTalentConfig,
    pub mail: MailConfig,
    pub world: WorldConfig,
    pub battle: BattleConfig,
    pub pay: PayConfig,
}

impl StaticConfig {
    /// 从数据库并行加载所有配置表
    ///
    /// 使用 `tokio::try_join!` 并行查询，加载失败时返回错误（服务应拒绝启动）。
    pub async fn load_from_db(pool: &sqlx::MySqlPool) -> anyhow::Result<Self> {
        let start = Instant::now();

        // 分两批并行加载，避免单次 try_join 参数过多
        let (activity, hero, building, tech, equip, item, task, shop) = tokio::try_join!(
            ActivityConfig::load(pool),
            HeroConfig::load(pool),
            BuildingConfig::load(pool),
            TechConfig::load(pool),
            EquipConfig::load(pool),
            ItemConfig::load(pool),
            TaskConfig::load(pool),
            ShopConfig::load(pool),
        )?;

        let (vip, skin, lord_equip, lord_talent, mail, world, battle, pay) = tokio::try_join!(
            VipConfig::load(pool),
            SkinConfig::load(pool),
            LordEquipConfig::load(pool),
            LordTalentConfig::load(pool),
            MailConfig::load(pool),
            WorldConfig::load(pool),
            BattleConfig::load(pool),
            PayConfig::load(pool),
        )?;

        let cfg = Self {
            activity, hero, building, tech, equip, item, task, shop,
            vip, skin, lord_equip, lord_talent, mail, world, battle, pay,
        };

        let elapsed = start.elapsed();
        info!(
            "StaticConfig loaded in {:.2}s — activity_plans={}, heroes={}, buildings={}, \
             techs={}, equips={}, items={}, tasks={}, shops={}, vips={}, skins={}, \
             lord_equips={}, lord_talents={}, mails={}, maps={}, npcs={}, mines={}, \
             battle_skills={}, pays={}",
            elapsed.as_secs_f64(),
            cfg.activity.plans.len(),
            cfg.hero.heroes.len(),
            cfg.building.buildings.len(),
            cfg.tech.tech_levels.len(),
            cfg.equip.equips.len(),
            cfg.item.props.len(),
            cfg.task.tasks.len(),
            cfg.shop.shops.len(),
            cfg.vip.vips.len(),
            cfg.skin.skins.len(),
            cfg.lord_equip.lord_equips.len(),
            cfg.lord_talent.talents.len(),
            cfg.mail.mails.len(),
            cfg.world.maps.len(),
            cfg.world.npcs.len(),
            cfg.world.mines.len(),
            cfg.battle.battle_skills.len(),
            cfg.pay.pays.len(),
        );

        // 基础校验
        cfg.validate();

        Ok(cfg)
    }

    /// 基础配置校验
    ///
    /// 校验失败仅告警，不阻止启动。
    fn validate(&self) {
        // 活动计划引用的 form_id 必须在 form_plans 中存在
        for (aid, form_ids) in &self.activity.activity_forms_idx {
            for fid in form_ids {
                if !self.activity.form_plans.contains_key(fid) {
                    warn!(
                        "Validation: activity_plan {} references form_id {} which is not in s_activity_form_plan",
                        aid, fid
                    );
                }
            }
        }

        // 活动任务引用的 form_id 必须在 form_plans 中存在
        for task in &self.activity.tasks {
            if !self.activity.form_plans.contains_key(&task.form_id) {
                warn!(
                    "Validation: activity_task {} references form_id {} which is not in s_activity_form_plan",
                    task.id, task.form_id
                );
            }
        }

        // 装备等级引用的 equipId 必须在 equips 中存在
        for lv in &self.equip.equip_levels {
            if !self.equip.equips.contains_key(&lv.equip_id) {
                warn!(
                    "Validation: equip_lv {} references equipId {} which is not in s_equip",
                    lv.id, lv.equip_id
                );
            }
        }

        // 商店商品引用的 shopId 必须在 shops 中存在
        for prop in &self.shop.shop_props {
            if let Some(sid) = prop.shop_id {
                if !self.shop.shops.contains_key(&sid) {
                    warn!(
                        "Validation: shop_prop {} references shopId {} which is not in s_shop",
                        prop.id, sid
                    );
                }
            }
        }
    }
}

/// 配置热加载管理器
///
/// 持有数据库连接池和 watch channel 发送端。
/// 调用 `reload()` 重新加载所有配置并广播给所有订阅者。
pub struct ConfigWatcher {
    pub db: sqlx::MySqlPool,
    pub tx: watch::Sender<Arc<StaticConfig>>,
}

impl ConfigWatcher {
    /// 创建 ConfigWatcher，初始加载配置
    ///
    /// 返回 (watcher, receiver)，receiver 可 clone 给各 Actor。
    pub async fn new(db: sqlx::MySqlPool) -> anyhow::Result<(Self, watch::Receiver<Arc<StaticConfig>>)> {
        let initial_config = StaticConfig::load_from_db(&db).await?;
        let (tx, rx) = watch::channel(Arc::new(initial_config));
        Ok((Self { db, tx }, rx))
    }

    /// 热加载：重新从数据库加载所有配置并广播
    pub async fn reload(&self) -> anyhow::Result<()> {
        info!("ConfigWatcher: reloading all static configs...");
        let start = Instant::now();
        let new_config = StaticConfig::load_from_db(&self.db).await?;
        let _ = self.tx.send(Arc::new(new_config));
        info!("ConfigWatcher: reload completed in {:.2}s", start.elapsed().as_secs_f64());
        Ok(())
    }
}
