# Phase 7：游戏协议兼容层

> 状态：待实现
> 前置依赖：Phase 1（Gateway 基础框架）
> 预估工期：2-3 周

---

## 一、目标

实现 Rust 服务端与现有客户端的完整协议兼容，确保客户端无需任何修改即可连接 Rust 版服务端。

---

## 二、现有协议体系分析

### 2.1 协议格式

Java 版使用 Protobuf 2.5，消息封装在 `Base` 消息中，通过 extensions 机制区分不同命令：

```protobuf
// Base.proto
message Base {
  required int32 command = 1;    // 命令号
  optional int32 code = 2;       // 错误码
  extensions 1000 to max;        // 所有业务消息通过 extension 挂载
}
```

每个具体消息通过 `extend Base { optional XxxRq ext = <cmd_id>; }` 挂载到 Base 上。

### 2.2 网络帧格式

```
┌──────────┬──────────┬──────────────────┐
│ 4 bytes  │ 4 bytes  │ N bytes          │
│ 总长度    │ 命令号    │ protobuf body    │
│ (大端)    │ (大端)    │ (Base 消息)      │
└──────────┴──────────┴──────────────────┘
```

### 2.3 proto 文件清单

当前 Rust proto 目录已包含从 Java 版迁移的所有 proto 文件：

| proto 文件 | 命令号范围 | 说明 |
|-----------|-----------|------|
| Game.proto | 1101-4406 | 登录、角色、任务、排行、GM |
| Activity.proto | 8001-8074, 8961-8966 | 运营活动 |
| World.proto | 2xxx-3xxx | 世界地图、行军、战斗 |
| Hero.proto | - | 将领系统 |
| Equip.proto | - | 装备系统 |
| Technology.proto | - | 科技系统 |
| Bag.proto | - | 背包系统 |
| Shop.proto | - | 商店系统 |
| Chat.proto | - | 聊天系统 |
| Social.proto | - | 社交系统 |
| Mail.proto | - | 邮件系统 |
| Camp.proto | - | 阵营系统 |
| Arena.proto | - | 竞技场 |
| Battle.proto | - | 战斗系统 |
| Combat.proto | - | 战斗详情 |
| Vip.proto | - | VIP 系统 |
| Pay.proto | - | 支付系统 |
| Skin.proto | - | 皮肤系统 |
| Guise.proto | - | 外观系统 |
| Wall.proto | - | 城墙系统 |
| Simulate.proto | - | 模拟系统 |
| Gameplay.proto | - | 玩法系统 |
| LordEquip.proto | - | 领主装备 |
| LordTalent.proto | - | 领主天赋 |
| Artifact.proto | - | 神器系统 |
| IntelBroker.proto | - | 情报系统 |
| OpinionLetterBox.proto | - | 意见箱 |
| Cross.proto | - | 跨服系统 |

---

## 三、核心挑战：proto2 extensions

### 3.1 问题

prost（Rust 的 protobuf 库）支持 proto2 语法，但对 `extensions` 的支持有限：
- prost 会将 extension 字段编码为未知字段
- 不会自动生成 extension 的 getter/setter
- 需要手动处理 extension 字段的编解码

### 3.2 解决方案

#### 方案 A：自定义编解码层（推荐，短期）

在 prost 生成的基础结构上，手动实现 extension 字段的编解码：

```rust
/// Base 消息的自定义编解码
pub struct GameMessage {
    pub command: i32,
    pub code: i32,
    /// 原始 extension 字段的字节数据
    pub extension_data: HashMap<u32, Vec<u8>>,
}

impl GameMessage {
    /// 从原始字节解码
    pub fn decode(data: &[u8]) -> Result<Self> {
        // 1. 解码 Base 的 command 和 code 字段
        // 2. 将剩余字段按 field_number 存入 extension_data
        // 3. 根据 command 确定具体的 extension field_number
    }
    
    /// 获取具体的请求消息
    pub fn get_request<T: prost::Message + Default>(&self, field_number: u32) -> Result<T> {
        let data = self.extension_data.get(&field_number)
            .ok_or(anyhow!("missing extension field {}", field_number))?;
        T::decode(data.as_slice()).map_err(Into::into)
    }
    
    /// 构建响应消息
    pub fn build_response<T: prost::Message>(command: i32, field_number: u32, msg: &T) -> Result<Vec<u8>> {
        // 编码 Base { command, code=0 } + extension { field_number: msg }
    }
}
```

#### 方案 B：proto3 重写 + Gateway 转换层（长期）

1. 将所有 proto2 文件重写为 proto3，用 `oneof` 替代 `extensions`
2. 在 Gateway 层实现 proto2 ↔ proto3 的转换
3. 服务端内部全部使用 proto3

#### 方案 C：使用 protobuf crate 替代 prost

`protobuf` crate（rust-protobuf）对 proto2 extensions 有更好的支持，但与 tonic 集成不如 prost 方便。可以：
- gRPC 内部通信用 prost + proto3
- 客户端协议编解码用 protobuf crate + proto2

### 3.3 推荐路径

短期用方案 A（自定义编解码层），长期迁移到方案 B。

---

## 四、命令号注册与路由

### 4.1 完整命令号表

需要在 `shared/src/cmd.rs` 中注册所有命令号，并标记路由目标（Home / World / Auth）：

```rust
pub enum CmdRoute {
    Auth,   // 认证相关
    Home,   // 玩家个人数据
    World,  // 世界地图
}

/// 命令号 → 路由目标的映射
pub fn cmd_route(cmd: u32) -> CmdRoute {
    match cmd {
        // 登录认证
        1101..=1108 => CmdRoute::Auth,
        
        // 玩家个人（Home）
        1109..=1206 => CmdRoute::Home,  // 角色数据、任务、排行等
        1175..=1194 => CmdRoute::Home,  // 限时任务、每日任务
        4401..=4406 => CmdRoute::Home,  // 惊喜奖励、玩家快照
        8001..=8074 => CmdRoute::Home,  // 活动系统
        8961..=8966 => CmdRoute::Home,  // 银行活动
        
        // 世界地图（World）
        2000..=3999 => CmdRoute::World, // 行军、战斗、地图
        
        _ => CmdRoute::Home, // 默认路由到 Home
    }
}
```

### 4.2 Gateway 编解码器

```rust
// crates/gateway/src/codec.rs 需要实现：

/// 帧编解码器（对应 Java Netty 的 LengthFieldBasedFrameDecoder）
pub struct GameCodec;

impl Decoder for GameCodec {
    type Item = GameFrame;
    type Error = anyhow::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<GameFrame>> {
        if src.len() < 8 { return Ok(None); }
        let total_len = src.get_u32() as usize;
        let cmd = src.get_u32();
        if src.len() < total_len - 8 { return Ok(None); }
        let body = src.split_to(total_len - 8).freeze();
        Ok(Some(GameFrame { cmd, body }))
    }
}

impl Encoder<GameFrame> for GameCodec {
    type Error = anyhow::Error;
    
    fn encode(&mut self, item: GameFrame, dst: &mut BytesMut) -> Result<()> {
        let total_len = 8 + item.body.len();
        dst.put_u32(total_len as u32);
        dst.put_u32(item.cmd);
        dst.put(item.body);
        Ok(())
    }
}
```

---

## 五、FunctionClientBase 体系兼容

### 5.1 Java 版 FunctionData 体系

Java 版通过 `GetRoleDataRs` 返回所有功能数据，每个功能模块实现 `FunctionEntity`，序列化为 `FunctionClientBase` 的 extension：

```protobuf
message FunctionClientBase {
  extensions 1 to max;
}

// 各模块通过 extension 挂载：
// ActivityFunction.ext = 25
// HeroFunction.ext = 3
// BuildingFunction.ext = 1
// ...
```

### 5.2 Rust 版实现

```rust
/// 功能模块注册表
pub struct FunctionRegistry {
    /// extension field_number → 模块名
    modules: HashMap<u32, &'static str>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(25, "activity");
        m.insert(3, "hero");
        m.insert(1, "building");
        // ... 注册所有模块
        Self { modules: m }
    }
}

/// GetRoleDataRs 的构建
impl PlayerActor {
    fn build_get_role_data_rs(&self) -> GetRoleDataRs {
        let mut function_bases = Vec::new();
        // 每个 PlayerSystem 序列化为 FunctionClientBase
        // 通过 extension field_number 区分模块
        function_bases.push(self.systems.activity.to_function_client_base(25));
        function_bases.push(self.systems.building.to_function_client_base(1));
        // ...
        GetRoleDataRs { function_base: function_bases }
    }
}
```

---

## 六、实现步骤

### Step 1：帧编解码（3 天）
- [ ] 实现 GameCodec（4字节长度头 + 4字节cmd + protobuf body）
- [ ] 验证与 Java 客户端的帧格式兼容性
- [ ] 处理大端/小端字节序

### Step 2：Base 消息 extension 编解码（1 周）
- [ ] 实现 GameMessage 自定义编解码
- [ ] 支持从 Base 消息中提取 extension 字段
- [ ] 支持构建带 extension 的 Base 响应消息
- [ ] 单元测试：用 Java 版编码的消息在 Rust 侧解码验证

### Step 3：命令号路由（3 天）
- [ ] 完善 cmd.rs 中的命令号枚举
- [ ] 实现 cmd → CmdRoute 路由表
- [ ] Gateway router 根据路由表分发到 Home/World

### Step 4：FunctionClientBase 兼容（3 天）
- [ ] 实现 FunctionClientBase extension 编解码
- [ ] 各 PlayerSystem 实现 to_function_client_base 方法
- [ ] GetRoleDataRs 完整构建

### Step 5：集成测试（3 天）
- [ ] 用 Java 客户端连接 Rust Gateway 验证登录流程
- [ ] 验证 GetRoleData 返回数据的正确性
- [ ] 验证活动协议的编解码兼容性

---

## 七、给 AI 的实现提示词

```
你是一个 Rust 游戏服务器开发者。请实现与 Java 客户端兼容的协议编解码层。

核心挑战：
Java 版使用 protobuf 2.5 的 extensions 机制，所有业务消息通过 extend Base 挂载。
prost 对 proto2 extensions 支持有限，需要自定义编解码。

网络帧格式：
- 4 字节大端总长度 + 4 字节大端命令号 + protobuf Base 消息体
- Base 消息包含 command(int32) + code(int32) + extensions(1000 to max)
- 具体业务消息通过 extension field_number 区分

实现要求：
1. GameCodec：tokio_util::codec 的 Encoder/Decoder 实现
2. GameMessage：自定义 Base 消息编解码，支持 extension 字段的提取和构建
3. 命令号路由：根据 cmd 分流到 Home/World/Auth
4. FunctionClientBase：支持 extension 编解码，用于 GetRoleDataRs

参考 proto 文件：
- Base.proto：Base 消息定义，extensions 1000 to max
- Game.proto：登录、角色、任务等命令（1101-4406）
- Activity.proto：活动命令（8001-8074）
- Common.proto：公共数据结构

测试方法：
用 Java protobuf 2.5 编码一条 BeginGameRq 消息，在 Rust 侧解码验证字段正确性。
```
