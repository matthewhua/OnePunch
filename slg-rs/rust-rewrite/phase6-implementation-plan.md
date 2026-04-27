# Phase 6 Activity Framework 实现步骤拆解

这份文档将 `phase6-activity-framework.md` 里的宏伟蓝图拆解为了 4 个带有详细**代码结构**及对应的 **Gemini Flash 提示词** 的可执行 Stage（[Stage 1] 到 [Stage 4]）。

您可以分批复制提示词（Prompt）交给大模型（Gemini 3.1 Flash）为您一次性生成对应包的代码架构骨架。

---

### [Stage 1] 活动基础骨架与协议兼容 (Base Framework & Protocol)
**目标**：建立 Home 服务端里 `ActivitySystem` 的核心数据流和 trait，它是接管玩家所有活动的抽象大脑。处理 proto2 的 extension 难题。

#### [NEW] `crates/home/src/systems/activity/types.rs`
#### [NEW] `crates/home/src/systems/activity/model.rs`
#### [NEW] `crates/home/src/systems/activity/mod.rs`
- **实现细节**:
  - 创建 `ActivityFormType` 枚举（剔除战令与社区跳转）和 `ActivityStage` 枚举。
  - 创建 `PersonalForm` 和 `CommonForm` 两个核心 trait，约束 `deserialize`、`to_client_pb` 以及每日 `tick` 的接口。
  - 创建 `ActivitySystem`，包含玩家私有数据（`PersonalActivity` map），使其继承 `PlayerSystem` trait 支持二进制存取。
- **Gemini Flash 提示词**:
  > "我们要开始实现 Rust 版的活动大框架 Phase 6 的 Stage 1 基础骨架。请帮我在 crates/home/src/systems/activity/ 目录下建立 types.rs, model.rs 和 mod.rs。
  > 1. 在 types.rs 中定义 `ActivityFormType` 枚举（参考 1-19 种玩法，忽略旧版的战令与跳转）。
  > 2. 在 model.rs 中定义 `PersonalForm` 和 `CommonForm` trait，要求支持反序列化、生成 PB 字节和生命周期事件（如 on_daily_tick）。
  > 3. 在 mod.rs 建立 `ActivitySystem` 结构体管理 `HashMap<i32, PersonalActivity>`，并实现 `PlayerSystem` trait 的 `load_from_bin` 和 `save_to_bin`，在内部留出 `handle_command` 对外路由函数的坑位。请展示代码骨架及对 proto2 extension 提供一层手动的解码封装思路（比如 `decode_form_extension` 函数）。"

---

### [Stage 2] 活动生命周期与全服实例 (Lifecycle & ActivityActor)
**目标**：活动是有时效和分阶段运作的，因此必须有一个全服共享的 Actor 负责根据配置文件按每秒进行滴答推进。

#### [NEW] `crates/home/src/actors/activity_actor.rs`
#### [NEW] `crates/home/src/systems/activity/lifecycle.rs`
- **实现细节**:
  - `ActivityActor` 保存全服公有数据（`GlobalActivityData`），如：全服统一生效的任务目标、排行榜大盘数据。
  - 通过 `tokio::time::interval(Duration::from_secs(1))` 每秒推进游戏时间轴。
  - 维护预展示 (PreDisplay) -> 开启 (Open) -> 结束展示 (EndDisplay) -> 关闭 的状态机流转。
- **Gemini Flash 提示词**:
  > "现在进行 Phase 6 的 Stage 2 生命周期搭建。请帮我编写 `crates/home/src/actors/activity_actor.rs` 与配套的 `lifecycle.rs`。
  > 1. 让 `ActivityActor` 管理全服级别的 `HashMap<i32, GlobalActivityData>`。
  > 2. 提供一个 event loop 的 `run` 方法，每秒 `tick()`，判断各个全服活动的 begin_time 和 end_time，进行状态切换或者阶段切换（类似最强领主每天一切换）。
  > 3. 能够接收外部消息 `ActivityMessage`，比如来自各玩家投递的 `UpdateRank` 积分上传消息或者提供由内部向外的 `GetGlobalActivity` 的 RPC 请求。无需关心所有细节，请以优雅清晰且容错的 async Actor 形式输出核心主干。"

---

### [Stage 3] 首批核心玩法模板跑通 (Initial Core Forms)
**目标**：有了架子后，先落地 4 种最经典的活动玩法充实 Form 集合，验证事件驱动（如做日常导致送奖积分）流程。

#### [NEW] `crates/home/src/systems/activity/forms/mod.rs`
#### [NEW] `crates/home/src/systems/activity/forms/sign.rs` (签到)
#### [NEW] `crates/home/src/systems/activity/forms/task.rs` (任务)
#### [NEW] `crates/home/src/systems/activity/forms/score.rs` (积分)
#### [NEW] `crates/home/src/systems/activity/forms/supreme_lord.rs` (最强领主)
- **实现细节**:
  - 这四者必须去各自实现 Stage 1 中的 `PersonalForm`。
  - 最简单的是签到表单 `SignForm`，仅记录领取标志位；最难的是 `SupremeLordForm` 因为存在阶段子排行榜。
- **Gemini Flash 提示词**:
  > "现在进行 Phase 6 的 Stage 3 首批具体玩法填充。请帮我在 `crates/home/src/systems/activity/forms/` 目录下建立 mod.rs 并实现四个经典的派生表单模块：sign.rs(首发签到), task.rs(任务), score.rs(积分奖励), supreme_lord.rs(阶段排行榜最强领主)。
  > 要求：它们都要作为 struct 去 impl `PersonalForm` trait，要求写出签到结构的内部字段表示（如领取了多少天），并实现 on_daily_tick 方法里的对应重置逻辑（比如跨天发奖状态）。给出具体代码示范和如何集成回外层 ActivitySystem 哈希表的枚举注入。"

---

### [Stage 4] 全套路由挂载与结算调度 (Settle & Command Mapping)
**目标**：完成拼图的最后一块。解决服务端与客户端之间几百上个不同的 proto 通讯与排行榜赛季派活下发问题。

#### [NEW] `crates/home/src/systems/activity/settle.rs`
#### [MODIFY] `crates/shared/src/cmd.rs`
- **实现细节**:
  - 定义 `SupremeLordStageSettleAward`、`RankSettleCommand` 之类的动作。当 Actor 检测到期时调用它们将奖品作为邮件（或是 direct insert 到玩家库里）。
  - 在公共头文件区把 `8001` 到 `8074` 范围的大量 cmd 路由绑定至活动模块处理函数下辖。
- **Gemini Flash 提示词**:
  > "现在进入 Phase 6 的最后一环：Stage 4 路由绑定与结算派奖。
  > 1. 请在 `shared/src/cmd.rs` 中预留和补充 8001~8074 这批关于客户端请求打卡、领奖、查询积分的路由号码。
  > 2. 请编写 `crates/home/src/systems/activity/settle.rs`，提供一个专门做结算（Settle）模块代码范例。例如向玩家发放排行榜最终名词后的奖励包的流程，该方法需要具有防重入性（不能多发），并在领取后把发奖状态更新至 `CommonActivityForm` 或者玩家个人数据中。请以安全稳健的 Rust 风格展示实现逻辑！"
