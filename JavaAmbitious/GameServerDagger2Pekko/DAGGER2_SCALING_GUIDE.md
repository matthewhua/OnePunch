# Dagger2 模块扩展指南

## 当前架构优势

### 1. 模块分组策略
```
GameServerComponent (顶层组件)
├── InfrastructureModule (基础设施聚合)
│   ├── GameServerModule (配置)
│   ├── ActorSystemModule (Actor系统)
│   └── DataAccessModule (数据访问)
└── BusinessModule (业务逻辑聚合)
    └── ServiceModule (业务服务)
```

### 2. 可扩展性
- **新增模块无需修改 `GameServerComponent`**
- 只需在对应的聚合模块中添加 `includes`
- 编译时检查依赖完整性

---

## 扩展示例

### 场景 1：新增商城系统

#### Step 1: 创建 ShopModule
```java
@Module
public class ShopModule {
    @Singleton
    @Provides
    static ShopService provideShopService(PlayerDAO playerDAO) {
        return new ShopServiceImpl(playerDAO);
    }
    
    @Singleton
    @Provides
    static ShopDAO provideShopDAO(SessionFactory sessionFactory) {
        return new ShopDAOImpl(sessionFactory);
    }
}
```

#### Step 2: 添加到 BusinessModule
```java
@Module(includes = {
    ServiceModule.class,
    ShopModule.class  // ← 只需添加这一行
})
public class BusinessModule {
}
```

#### Step 3: (可选) 在 Component 中暴露
```java
public interface GameServerComponent {
    // 仅在需要频繁访问时才暴露
    ShopService getShopService();
}
```

**✅ 优势**：`GameServerComponent` 的 `@Component` 注解无需修改！

---

### 场景 2：新增公会系统（包含多个模块）

#### Step 1: 创建 GuildModule 聚合
```java
@Module(includes = {
    GuildServiceModule.class,
    GuildDAOModule.class,
    GuildEventModule.class
})
public class GuildModule {
    // 聚合模块
}
```

#### Step 2: 添加到 BusinessModule
```java
@Module(includes = {
    ServiceModule.class,
    ShopModule.class,
    GuildModule.class  // ← 添加公会系统
})
public class BusinessModule {
}
```

---

### 场景 3：新增数据库（如 Redis）

#### Step 1: 创建 CacheModule
```java
@Module
public class CacheModule {
    @Singleton
    @Provides
    static RedisClient provideRedisClient(GameServerConfig config) {
        return new RedisClient(config.getRedisUrl());
    }
}
```

#### Step 2: 添加到 InfrastructureModule
```java
@Module(includes = {
    GameServerModule.class,
    ActorSystemModule.class,
    DataAccessModule.class,
    CacheModule.class  // ← 添加缓存模块
})
public class InfrastructureModule {
}
```

---

## 按需暴露策略

### 问题：所有依赖都暴露会导致 Component 接口膨胀

### 解决方案：使用 `Provider<T>` 延迟获取

#### 示例：不常用的 DAO 不暴露

```java
// ❌ 不推荐：每个 DAO 都暴露
public interface GameServerComponent {
    PlayerDAO getPlayerDAO();
    ShopDAO getShopDAO();
    GuildDAO getGuildDAO();
    ItemDAO getItemDAO();
    // ... 100 个 DAO 方法
}

// ✅ 推荐：只暴露核心 DAO
public interface GameServerComponent {
    PlayerDAO getPlayerDAO();  // 高频使用
    
    // 其他 DAO 通过 Provider 获取
    Provider<ShopDAO> shopDAOProvider();
    Provider<GuildDAO> guildDAOProvider();
}

// 使用方式
ShopDAO shopDAO = component.shopDAOProvider().get();
```

---

## 性能对比

### 当前架构 vs 原始架构

| 指标 | 原始架构 | 模块分组架构 |
|------|---------|-------------|
| **编译时间** | 基准 | +5%（多一层 Module） |
| **运行时开销** | 零 | 零（编译时展开） |
| **代码维护性** | ★★☆☆☆ | ★★★★★ |
| **扩展难度** | 每次改 Component | 只改聚合 Module |

---

## 最佳实践

### 1. 模块分类原则
- **InfrastructureModule**：基础设施（不依赖业务逻辑）
- **BusinessModule**：业务逻辑（依赖基础设施）
- **ActorModule**：Actor 工厂（依赖业务逻辑）

### 2. 暴露策略
- **高频访问**：直接暴露 getter（如 `PlayerDAO`）
- **低频访问**：使用 `Provider<T>`（如 `shopDAOProvider()`）
- **内部依赖**：完全不暴露（仅在 Module 内部注入）

### 3. 何时创建新的聚合 Module
- **超过 5 个相关模块**：创建聚合（如 `GuildModule` 聚合多个公会模块）
- **跨多个功能域**：按领域拆分（如 `PvPModule`、`PvEModule`）

### 4. 避免循环依赖
```java
// ❌ 错误：循环依赖
@Module(includes = {BusinessModule.class})
class InfrastructureModule {}

@Module(includes = {InfrastructureModule.class})
class BusinessModule {}

// ✅ 正确：单向依赖
InfrastructureModule → BusinessModule → ActorModule
```

---

## 未来扩展建议

### 如果模块数量超过 20 个

考虑引入**领域驱动设计（DDD）**：

```
GameServerComponent
├── PlayerDomainModule (玩家域)
│   ├── PlayerModule
│   ├── BackpackModule
│   └── SkillModule
├── SocialDomainModule (社交域)
│   ├── GuildModule
│   ├── FriendModule
│   └── ChatModule
└── EconomyDomainModule (经济域)
    ├── ShopModule
    ├── AuctionModule
    └── TradeModule
```

### 如果需要动态模块加载

考虑切换到 **Spring Boot**，Dagger2 不支持运行时动态加载。

---

## 总结

**当前架构解决了什么问题**：
1. ✅ `GameServerComponent` 不再随模块数量膨胀
2. ✅ 新增模块只需修改聚合 Module，无需改顶层 Component
3. ✅ 保留了 Dagger2 编译时检查的优势
4. ✅ 通过 `Provider<T>` 避免暴露所有依赖

**何时需要重构**：
- 聚合 Module 超过 10 个 → 考虑领域分组
- 需要动态加载模块 → 考虑切换到 Spring
- 团队新人学习成本高 → 考虑切换到 Spring

**适用场景**：
- ✅ 高性能游戏服（启动快、运行时零开销）
- ✅ 模块数量在 50 个以内
- ✅ 团队熟悉 Dagger2
