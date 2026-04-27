# Phase 10：剩余功能系统清单

> 状态：待实现
> 前置依赖：Phase 3（Home Service）、Phase 4（World Service）
> 说明：按优先级排列，每个系统可独立实现

---

## 一、当前已实现 vs 待实现对照

### 1.1 Rust 已实现（骨架级别）

| 模块 | crate | 状态 | 说明 |
|------|-------|------|------|
| Gateway TCP 监听 | gateway | ✅ 骨架 | 基础 TCP accept + handler |
| Gateway 编解码 | gateway/codec | ✅ 骨架 | 帧编解码基础实现 |
| Auth 登录验证 | auth | ✅ 基础 | gRPC Login + ValidateToken |
| Auth Session 管理 | auth/session | ✅ 基础 | 内存 session（待接 Redis） |
| PlayerActor | home/actor | ✅ 骨架 | 消息循环 + RoleLogin/GetRoleData |
| PlayerManager | home/manager | ✅ 基础 | spawn_actor + 在线管理 |
| PlayerSystem trait | home/systems | ✅ 定义 | load_from_bin / save_to_bin |
| BuildingSystem | home/systems/building | ✅ 骨架 | 空实现 |
| SkinSystem | home/systems/skin | ✅ 骨架 | 空实现 |
| World gRPC 服务 | world/service | ✅ 骨架 | JoinMap 空实现 |
| 地图 AOI | world/map/aoi | ✅ 骨架 | 待实现 |
| 地图格子 | world/map/grid | ✅ 骨架 | 待实现 |
| 行军系统 | world/march | ✅ 骨架 | 待实现 |
| Proto 文件 | proto | ✅ 完整 | 所有 proto 文件已迁移 |
| 共享库 | shared | ✅ 基础 | config, db, error, cmd |

### 1.2 待实现系统（按优先级）

---

## 二、P0 — 核心流程（必须先做）

### 2.1 协议兼容层（Phase 7）
- proto2 extensions 编解码
- Base 消息自定义编解码
- FunctionClientBase 体系
- 详见 `phase7-protocol-compat.md`

### 2.2 静态配置加载（Phase 9）
- 从 MySQL s_ 表加载配置
- 配置热加载机制
- 详见 `phase9-static-config.md`

### 2.3 玩家数据持久化
- p_account 表读写
- p_data 表 blob 字段读写（各 FunctionEntity）
- 异步定时存盘（5 分钟 + 下线存盘）
- dirty 标记优化

---

## 三、P1 — Home Service 功能系统

每个系统对应 Java 的一个 FunctionEntity + Handler + Service：

| 系统 | Java 对应 | proto | 优先级 | 说明 |
|------|-----------|-------|--------|------|
| 将领系统 | HeroFunction + HeroHandler | Hero.proto | P1 | 升级、升星、技能、装备 |
| 背包系统 | BackpackFunction + BackpackHandler | Bag.proto | P1 | 道具使用、合成、分解 |
| 科技系统 | TechFunction + TechHandler | Technology.proto | P1 | 研究、加速 |
| 装备系统 | EquipFunction + EquipHandler | Equip.proto | P1 | 锻造、强化、镶嵌 |
| 建筑系统 | BuildingFunction + BuildingHandler | Simulate.proto | P1 | 升级、拆除、加速 |
| 任务系统 | MissionFunction + MissionHandler | Game.proto | P1 | 主线、日常、成长、限时 |
| VIP 系统 | VipFunction + VipHandler | Vip.proto | P2 | VIP 等级、特权 |
| 商店系统 | ShopFunction + ShopHandler | Shop.proto | P2 | 购买、刷新 |
| 邮件系统 | MailFunction + MailHandler | Mail.proto | P2 | 收发邮件、附件领取 |
| 聊天系统 | ChatFunction + ChatHandler | Chat.proto | P2 | 世界/阵营/私聊 |
| 社交系统 | SocialFunction + SocialHandler | Social.proto | P2 | 好友、黑名单 |
| 皮肤系统 | SkinFunction + SkinHandler | Skin.proto | P3 | 皮肤解锁、穿戴 |
| 外观系统 | GuiseFunction + GuiseHandler | Guise.proto | P3 | 头像、头像框 |
| 领主装备 | LordEquipFunction | LordEquip.proto | P3 | 领主专属装备 |
| 领主天赋 | LordTalentFunction | LordTalent.proto | P3 | 天赋树 |
| 神器系统 | ArtifactFunction | Artifact.proto | P3 | 神器升级 |
| 情报系统 | IntelBrokerFunction | IntelBroker.proto | P3 | 情报收集 |
| 城墙系统 | WallFunction + WallHandler | Wall.proto | P3 | 城墙防御 |
| 支付系统 | PayFunction + PayHandler | Pay.proto | P3 | 充值、订单 |

---

## 四、P1 — World Service 功能系统

| 系统 | Java 对应 | 优先级 | 说明 |
|------|-----------|--------|------|
| 世界地图 | WorldMapService | P1 | 地图格子、实体管理 |
| AOI 系统 | AOI 广播 | P1 | 视野管理、事件广播 |
| 行军系统 | MarchService | P1 | 派兵、召回、行军计算 |
| 部队系统 | TroopService | P1 | 部队创建、属性计算 |
| 战斗触发 | WorldFightService | P1 | 攻击玩家/NPC 触发战斗 |
| 采集系统 | GatherService | P2 | 资源点采集 |
| NPC 据点 | NpcService | P2 | NPC 刷新、攻打 |
| 阵营系统 | CampService | P2 | 阵营战、领地 |
| 排行榜 | RankService | P2 | 全服排行 |
| 城池实体 | CityEntity | P2 | 玩家城池在地图上的表现 |

---

## 五、P2 — 活动系统（Phase 6）

详见 `phase6-activity-framework.md`

---

## 六、P2 — 事件系统（Phase 8）

详见 `phase8-event-system.md`

---

## 七、P3 — 跨服系统

| 系统 | Java 对应 | 说明 |
|------|-----------|------|
| 跨服匹配 | CrossService | 跨服竞技匹配 |
| 跨服战斗 | CrossFightService | 跨服战斗逻辑 |
| 跨服通信 | Dubbo RPC → gRPC | 服务间通信 |

---

## 八、P3 — 战斗系统

| 系统 | Java 对应 | 说明 |
|------|-----------|------|
| 战斗引擎 | FightService | 回合制战斗计算 |
| 战斗报告 | BattleReport | 战报生成和回放 |
| 里程碑 Boss | MilestoneRaiderBoss | 世界 Boss 战斗 |

---

## 九、建议实现顺序

```
Phase 7（协议兼容）──→ Phase 9（配置加载）──→ 玩家持久化
        │                                        │
        ▼                                        ▼
Phase 3 补全（Home 各系统骨架）──→ Phase 8（事件系统）──→ Phase 6（活动框架）
        │
        ▼
Phase 4 补全（World 地图/行军/战斗）──→ 跨服 ──→ 集成测试
```

核心路径：协议兼容 → 配置加载 → 持久化 → 事件系统 → 活动框架

每个 Home 系统可以独立并行开发，只要 PlayerSystem trait 和事件系统就绪。
