# 静态配置系统需求文档

> 对应 Phase：Phase 9
> 优先级：P0（所有业务逻辑的数据驱动基础）
> 预估工期：1-2 周

---

## 一、概述

游戏的所有数值、规则、奖励等都由策划配置在 MySQL 的 `s_` 前缀表中。服务端启动时批量加载这些配置到内存，运行时只读访问。对应 Java 版的 `StaticDataMgr` 体系。

---

## 二、功能需求

### 2.1 配置加载框架

#### FR-SC-001：启动时批量加载
- 服务启动时从 MySQL 批量加载所有 `s_` 表
- 支持并行加载（多个表同时查询）
- 加载失败时服务拒绝启动，输出明确的错误信息
- 加载完成后通过 `Arc<StaticConfig>` 共享给所有 Actor

#### FR-SC-002：配置结构体定义
- 每种配置表对应一个 Rust 结构体
- 使用 `sqlx::FromRow` 自动映射数据库行
- 配置间的关联关系在加载后建立索引（如 activityId → formIds）

#### FR-SC-003：配置热加载
- 支持 GM 命令触发配置热加载
- 热加载生成新的 `Arc<StaticConfig>`，通过 `watch::channel` 广播
- 各 Actor 通过 `watch::Receiver` 自动获取最新配置
- 热加载期间不影响正在处理的请求（旧配置继续生效直到切换）

#### FR-SC-004：配置校验
- 加载后执行基础校验：必填字段非空、ID 唯一、外键引用有效
- 校验失败时记录详细错误日志，但不阻止启动（仅告警）
- 提供 `validate()` 方法供 GM 工具调用

### 2.2 核心配置表

#### FR-SC-005：活动配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_activity_plan | StaticActivityPlan | 活动计划：开启条件、时间、循环 |
| s_activity_form_plan | StaticActivityFormPlan | 玩法计划：formId → formType |
| s_activity_cycle | StaticActivityCycle | 活动周期/赛季 |
| s_activity_form_sign | StaticActivityFormSign | 签到配置 |
| s_activity_form_task | StaticActivityFormTask | 任务配置 |
| s_activity_form_rank | StaticActivityFormRank | 排行榜配置 |
| s_activity_form_supreme_lord | StaticActivityFormSupremeLord | 最强领主配置 |

#### FR-SC-006：将领配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_hero | StaticHero | 将领基础属性 |
| s_hero_skill | StaticHeroSkill | 将领技能 |
| s_hero_star | StaticHeroStar | 将领升星 |
| s_hero_level | StaticHeroLevel | 将领等级经验 |

#### FR-SC-007：建筑配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_building | StaticBuilding | 建筑属性 |
| s_building_upgrade | StaticBuildingUpgrade | 建筑升级消耗和时间 |

#### FR-SC-008：科技配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_technology | StaticTechnology | 科技属性 |
| s_technology_level | StaticTechnologyLevel | 科技等级消耗 |

#### FR-SC-009：道具配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_item | StaticItem | 道具基础信息 |
| s_item_compose | StaticItemCompose | 道具合成配方 |

#### FR-SC-010：任务配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_mission | StaticMission | 任务定义 |
| s_chapter_mission | StaticChapterMission | 章节任务 |
| s_daily_mission | StaticDailyMission | 日常任务 |

#### FR-SC-011：世界地图配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_world_map | StaticWorldMap | 地图格子配置 |
| s_npc | StaticNpc | NPC 配置 |
| s_resource_point | StaticResourcePoint | 资源点配置 |

#### FR-SC-012：装备配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_equip | StaticEquip | 装备基础属性 |
| s_equip_upgrade | StaticEquipUpgrade | 装备强化 |

#### FR-SC-013：商店配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_shop | StaticShop | 商店商品 |
| s_shop_refresh | StaticShopRefresh | 商店刷新规则 |

#### FR-SC-014：VIP 配置
| 表名 | 对应结构体 | 说明 |
|------|-----------|------|
| s_vip | StaticVip | VIP 等级特权 |

### 2.3 配置索引

#### FR-SC-015：二级索引构建
- 加载后自动构建常用的查询索引：
  - `formType → Vec<formId>`（活动玩法类型索引）
  - `heroId → Vec<skillId>`（将领技能索引）
  - `buildingType → Vec<buildingId>`（建筑类型索引）
  - `missionType → Vec<missionId>`（任务类型索引）
- 索引在热加载时重建

---

## 三、非功能需求

### 3.1 性能
- 全量加载时间 < 5 秒（所有配置表）
- 配置查询延迟 < 100ns（内存 HashMap 查找）
- 热加载时间 < 3 秒

### 3.2 内存
- 配置数据内存占用 < 100MB
- 热加载期间短暂存在两份配置（旧 + 新），峰值 < 200MB

---

## 四、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| StaticConfig 容器 | ✅ 基础 | 仅有 ActivityConfig |
| ConfigWatcher | ✅ 基础 | watch channel 已实现 |
| ActivityConfig | ⚠️ 骨架 | 结构定义，查询逻辑待实现 |
| 其他配置 | ❌ 缺失 | 将领、建筑、科技等全部缺失 |
| 配置校验 | ❌ 缺失 | 需要实现 |
| 并行加载 | ❌ 缺失 | 当前串行加载 |
| 二级索引 | ❌ 缺失 | 需要实现 |

---

## 五、数据库表结构参考

需要从 Java 版的 MyBatis Mapper 和实体类中提取完整的表结构。关键参考文件：
- `StaticActivityConfigMgr.java`
- `StaticHeroMgr.java`
- `StaticBuildingMgr.java`
- 各 `s_` 表的 DDL

---

## 六、配置项

```toml
[static_config]
# 配置加载超时
load_timeout_secs = 30
# 是否启用配置校验
validate_on_load = true
# 热加载最小间隔（防止频繁刷新）
reload_cooldown_secs = 10
```
