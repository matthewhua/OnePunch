# Phase 7：游戏协议兼容层执行目标与步骤

该计划梳理了 Phase 7 的具体代码落地步骤，以便后续工具能够针对性地进行分步修改。

## 背景

为了全兼容 Java 客户端，我们需要处理 Protobuf 2.5 `extensions` 机制所带来的差异，并在网关中进行正确的拦截与编解码分发。本项目已经具备从 Java 迁移过来的所有 proto 文件以及使用 `prost` 进行代码生成的流程。

## 方案设计与改动清单

我们将分阶段修改，确保各服务逐步适应兼容。

---

### 第一阶段：编解码器 (Codec) 与消息结构增强 (`GameCodec` & `GameMessage`)

我们需要在应用层处理 Proto2 的 extensions 概念。由于 `prost` 默认不支持直接以特定结构体化读取 extension，我们将基于解码出来的 `Base` 消息和其底层二进制数据做一层封装：`GameMessage`。

#### [NEW] `crates/shared/src/msg.rs`

创建通用的游戏协议消息包装层 `GameMessage`：

1. 提供一个结构体封装来自网络层解析好的 `cmd` 和原生 `payload`。
2. 定义 `GameMessage::decode` 接口，它将从字节切片中解析出 `slg::Base`。
3. 定义 `GameMessage::get_request<T: prost::Message + Default>` 方法：基于 `cmd` 推导出的 `field_number`，从 `slg::Base` 的未知字段 (或直接在解析阶段隔离出来) 中反序列化出扩展消息。由于 `prost` 可能没有自动抓取未知字段的方法，可能需要我们在外层包装中直接通过特定规则或将 extension 当作单独剥离的数据。
4. **注意**：Phase 7 原文档建议维护一个 `HashMap<u32, Vec<u8>>` 为 extension_data。可以通过解析 Protobuf wire-format 来跳过 Base 字段提取 extension，或使用现成的库包装。

#### [MODIFY] `crates/gateway/src/codec.rs`

调整解码器，确保匹配真正的客户端帧结构：

1. **帧格式确认**：4 字节大端总长度 + 4 字节大端命令号 + Protobuf (Base 消息)。
2. （当前逻辑基本符合，总长度为 payload 加上 4 bytes cmd，我们需要检查现在的 `GameCodec` 是否需要同步将数据重构为包裹 `GameMessage`的抽象，而不是仅仅暴露裸数据）。

---

### 第二阶段：命令号注册与多路复发 (`CmdRoute` 表)

对 `cmd.rs` 增加基于 `Cmd ID` 的服务路由特性。这允许网关精确知道该把收到的一条请求推送到具体哪个服务进程 (或 Actor)（如 Auth、Home、World 等）。

#### [MODIFY] `crates/shared/src/cmd.rs`

1. 补全缺失的游戏核心指令：登录、行军、活动、世界地图等命令。
2. 定义路由目标枚举：
   ```rust
   pub enum CmdRoute {
       Auth,
       Home,
       World,
   }
   ```
3. 实现映射函数：
   ```rust
   pub fn cmd_route(cmd: u32) -> CmdRoute { ... }
   ```
   - 1101 ~ 1108 -> Auth
   - 1109 ~ 1206 / 1175 ~ 1194 / 8000+ -> Home
   - 2000 ~ 3999 -> World

---

### 第三阶段：功能数据序列化封装 (`FunctionClientBase`)

兼容 `FunctionClientBase` 要求在获取角色数据 (`GetRoleDataRs`) 或者单项数据同步时，按照对应的模块 ID 装填其 `Protobuf Message` 到 extension 字段内部。

#### [NEW] `crates/home/src/actors/function_registry.rs` 或 `crates/home/src/systems/function_client.rs`

1. 为 Home Actor 体系设计一个模块化打包的 Traits：
   ```rust
   pub trait ToFunctionClientBase {
       fn to_function_client_base(&self, ext_id: u32) -> slg::FunctionClientBase;
   }
   ```
2. 模块分配表定义 (如 ActivityFunction=25, HeroFunction=3)。

#### [MODIFY] `crates/home/src/actors/player_actor.rs` 或相关的登录初始化逻辑

1. 改造组装 `GetRoleDataRs` 的流程。在登录完成后，遍历各个 System 挂载相应的功能树数据到 `GetRoleDataRs.function_base` 列表。

---

## 执行步骤分配表

*准备交给 Flash/后续执行循环的任务序列：*

1. **Task 1**: 编写 `crates/shared/src/msg.rs` 实现 `GameMessage` (包含基于 Prost/Wire 格式对 Proto2 extension 字段的解析逻辑)。
2. **Task 2**: 完善 `crates/shared/src/cmd.rs` 添加路由枚举 `CmdRoute` 及 `cmd_route()` 函数。
3. **Task 3**: 验证并调整 `crates/gateway/src/codec.rs`，确保封包解包符合：长度+命令号+Base消息，并和 `GameMessage` 结合。
4. **Task 4**: 实现 `FunctionClientBase` 的拓展支持机制 (在 Home Crate 中构建 `ToFunctionClientBase` trait 并配置常量)。
5. **Task 5**: 编写测试用例跑通客户端的一帧数据。

## 待确认项 (User Review)

> [!WARNING]
> **Protobuf Extension 的具体解析方案设计**：由于我们已经使用了 `prost` 库，而 prost 对 proto2 的未知字段与 extensions 在高版本中需要开启特殊 flag 或手动解析 wire tag。是否同意我们在 `msg.rs` 中手写一套微小的前缀解析从二进制流中抽离 extension 的 payload？或者您是否倾向于将协议栈整体换到 `rust-protobuf`（其默认支持 proto2 extension）仅用于网关层？
> 
> *推荐采用：直接手搓简单的 Wire Decoder 在 msg 层提取 field tag，保持使用 prost，符合短期快速落地方案。*

请审核并确认计划，如果有修改意见请提出，如无问题，则可以开始交由系统分步骤逐步落地。
