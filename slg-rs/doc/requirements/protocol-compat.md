# 协议兼容层需求文档

> 对应 Phase：Phase 7
> 优先级：P0（与客户端对接的前提）
> 预估工期：2-3 周

---

## 一、概述

Java 版客户端使用 Protobuf 2.5 的 `extensions` 机制封装所有业务消息。Rust 侧使用的 prost 库对 proto2 extensions 支持有限，需要实现自定义的编解码层，确保客户端无需任何修改即可连接 Rust 服务端。

---

## 二、功能需求

### 2.1 Base 消息编解码

#### FR-PC-001：Base 消息解码
- 从 protobuf 二进制流中解码 `Base` 消息
- 提取 `command`（int32）和 `code`（int32）基础字段
- 将 extension 区域（field_number >= 1000）的原始字节按 field_number 分组存储
- 支持嵌套 extension（如 ActivityFormPb 内部的 extension）

#### FR-PC-002：Base 消息编码
- 构建包含 command、code 和 extension 字段的 Base 响应消息
- 将具体的响应消息编码为 extension 字段，写入正确的 field_number
- 编码结果必须与 Java protobuf 2.5 编码的结果二进制兼容

#### FR-PC-003：Wire Format 解析
- 实现 protobuf wire format 的底层解析器
- 支持解析 varint、fixed32、fixed64、length-delimited 四种 wire type
- 能够按 field_number 提取任意字段的原始字节

### 2.2 命令号 → Extension 映射

#### FR-PC-004：命令号注册表
- 维护完整的命令号 → extension field_number 映射表
- 覆盖所有 proto 文件中定义的 `extend Base` 声明
- 支持运行时查询：给定 cmd，返回对应的 request/response field_number

#### FR-PC-005：命令号路由表
- 完善 `CmdRoute` 枚举和路由函数
- 覆盖所有命令号范围：
  - 1001-1108：Auth
  - 1109-1206：Home（角色、任务、排行）
  - 2000-3999：World（行军、战斗、地图）
  - 4401-4406：Home（惊喜奖励、快照）
  - 8001-8074：Home（活动系统）
  - 8961-8966：Home（银行活动）
  - 9001：系统（心跳）

### 2.3 FunctionClientBase 兼容

#### FR-PC-006：FunctionClientBase 编码
- 实现 `FunctionClientBase` 的 extension 编码
- 每个 PlayerSystem 通过 `ToFunctionClientBase` trait 序列化为带 extension 的 FunctionClientBase
- 模块 → extension field_number 映射表：
  - ActivityFunction = 25
  - HeroFunction = 3
  - BuildingFunction = 1
  - BackpackFunction = 2
  - TechFunction = 4
  - EquipFunction = 5
  - MissionFunction = 6
  - VipFunction = 7
  - ShopFunction = 8
  - MailFunction = 9
  - SkinFunction = 10
  - 等等（需从 Java 源码中提取完整映射）

#### FR-PC-007：GetRoleDataRs 构建
- 登录完成后，遍历所有 PlayerSystem
- 每个系统序列化为 FunctionClientBase
- 组装为 `GetRoleDataRs { function_base: Vec<FunctionClientBase> }`
- 客户端能正确解析每个模块的数据

### 2.4 特殊协议处理

#### FR-PC-008：活动 Extension 嵌套
- `ActivityFormPb` 内部也使用 extensions（`extensions 10 to 100`）
- 需要二级 extension 解码：先从 Base 提取 ActivityFormPb，再从 ActivityFormPb 提取具体玩法数据
- 每种 ActivityFormType 对应不同的 extension field_number

#### FR-PC-009：服务端推送消息
- 服务端主动推送消息给客户端（如战报、邮件通知、活动开启）
- 推送消息也需要封装为 Base + extension 格式
- 推送不需要 request_id，但需要正确的 command 号

---

## 三、技术方案

### 3.1 推荐方案：自定义 Wire Format 解析器

```rust
/// 游戏消息封装
pub struct GameMessage {
    pub command: i32,
    pub code: i32,
    /// extension field_number → 原始字节
    pub extensions: HashMap<u32, Vec<u8>>,
}

impl GameMessage {
    /// 从 Base 消息的原始字节解码
    pub fn decode(raw: &[u8]) -> Result<Self>;
    
    /// 提取指定 field_number 的 extension 并反序列化
    pub fn get_extension<T: prost::Message + Default>(&self, field_number: u32) -> Result<T>;
    
    /// 构建响应消息的原始字节
    pub fn encode_response(command: i32, code: i32, field_number: u32, msg: &impl prost::Message) -> Result<Vec<u8>>;
}
```

### 3.2 长期方案：proto3 迁移

- 将所有 proto2 文件重写为 proto3，用 `oneof` 替代 `extensions`
- Gateway 层实现 proto2 ↔ proto3 转换
- 服务端内部全部使用 proto3
- 此方案工期较长，建议作为后续优化

---

## 四、验证标准

### 4.1 兼容性测试
- 用 Java protobuf 2.5 编码一条 `BeginGameRq`，在 Rust 侧解码验证字段正确
- 用 Rust 编码一条 `BeginGameRs`，在 Java 侧解码验证字段正确
- 覆盖至少 20 种不同命令的编解码测试

### 4.2 性能测试
- 单条消息编解码延迟 < 10μs
- 编解码吞吐 > 1M msg/s

---

## 五、当前实现差距

| 功能 | 当前状态 | 差距 |
|------|---------|------|
| GameMessage 结构 | ⚠️ 定义 | 需要实现 wire format 解析 |
| Wire Format 解析器 | ❌ 缺失 | 核心工作量 |
| 命令号注册表 | ⚠️ 部分 | 仅 15 个，需补全 200+ |
| FunctionClientBase | ⚠️ trait 定义 | 需要实现具体编码逻辑 |
| 活动 Extension 嵌套 | ❌ 缺失 | 需要二级解码 |
| 兼容性测试 | ❌ 缺失 | 需要 Java 对照测试 |
