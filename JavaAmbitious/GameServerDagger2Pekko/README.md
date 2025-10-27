# 游戏服务器开发：Dagger2 + Pekko 实战教程

## 📋 目录

1. [技术关联说明](#技术关联说明)
2. [环境搭建步骤](#环境搭建步骤)
3. [核心实战模块](#核心实战模块)
4. [线程安全方案](#线程安全方案)
5. [避坑指南](#避坑指南)
6. [最佳实践](#最佳实践)
7. [快速开始](#快速开始)

---

## 技术关联说明

### 为什么游戏服需要同时使用 Dagger2 和 Pekko？

#### 问题背景

在游戏服务器开发中，我们面临两个核心挑战：

**挑战 1：Actor 依赖管理混乱**

```java
// ❌ 传统做法：构造器地狱
public class PlayerActor extends AbstractActor {
    private PlayerDAO playerDAO;
    private SkillService skillService;
    private MapService mapService;
    private CacheService cacheService;
    private LogService logService;
    // ... 还有更多依赖
    
    public PlayerActor(
        PlayerDAO playerDAO,
        SkillService skillService,
        MapService mapService,
        CacheService cacheService,
        LogService logService,
        // ... 还有更多参数
    ) {
        // 手动赋值，容易出错
    }
}

// 创建 Actor 时需要手动组装所有依赖
PlayerActor actor = new PlayerActor(
    new PlayerDAOImpl(dataSource),
    new SkillServiceImpl(),
    new MapServiceImpl(),
    new CacheServiceImpl(),
    new LogServiceImpl()
);
```

**问题：**
- 依赖关系复杂，容易遗漏或传错
- 每个 Actor 创建时都要重复组装依赖
- 修改依赖时需要改动多个地方
- 单元测试时难以 Mock 依赖

**挑战 2：Actor 实例化时的线程安全问题**

```java
// ❌ 不安全的做法
public class GameServer {
    private static PlayerDAO playerDAO = new PlayerDAOImpl();  // 全局单例
    
    public ActorRef createPlayerActor(long playerId) {
        // 多个线程并发调用时，可能导致竞态条件
        return actorSystem.actorOf(
            Props.create(PlayerActor.class, playerId, playerDAO),
            "player-" + playerId
        );
    }
}
```

---

### Dagger2 的作用

**Dagger2 是编译时依赖注入框架，解决方案：**

1. **统一管理依赖**
   - 在 `GameServerComponent` 中定义所有依赖
   - 编译时生成依赖图，避免运行时错误

2. **确保单例性**
   - `@Singleton` 注解确保 ActorSystem、DAO、Service 只创建一次
   - 所有 Actor 共享同一个实例，避免重复创建

3. **自动依赖注入**
   - Actor 工厂已注入所有依赖
   - 调用方只需 `factory.create(playerId)`，无需手动传递

4. **编译时检查**
   - 如果依赖缺失，编译时就会报错
   - 避免运行时的 `NullPointerException`

---

### Pekko 的作用

**Pekko 是 Actor 框架，提供：**

1. **并发模型**
   - 每个 Actor 是单线程的
   - 不同 Actor 可以并行执行
   - 通过消息队列通信，避免竞态条件

2. **生命周期管理**
   - Actor 的创建、运行、销毁都由框架管理
   - 支持监督策略，自动重启失败的 Actor

3. **分布式支持**
   - Pekko Cluster 支持多节点部署
   - ActorRef 可以跨节点通信

---

### 两者结合的优势

```
┌─────────────────────────────────────────────────────────┐
│                   游戏服务器架构                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Dagger2 依赖注入容器（编译时生成）              │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │ @Singleton                                 │  │   │
│  │  │ - ActorSystem                              │  │   │
│  │  │ - PlayerDAO (数据库连接池)                 │  │   │
│  │  │ - SkillService (技能系统)                  │  │   │
│  │  │ - MapService (地图系统)                    │  │   │
│  │  │ - PlayerActorFactory                       │  │   │
│  │  │ - MapActorFactory                          │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
│                        ↓                                  │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Pekko ActorSystem（运行时执行）                │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │ 玩家 Actor 1 (单线程)                      │  │   │
│  │  │ 玩家 Actor 2 (单线程)                      │  │   │
│  │  │ 玩家 Actor 3 (单线程)                      │  │   │
│  │  │ ...                                        │  │   │
│  │  │ 地图 Actor 1 (单线程)                      │  │   │
│  │  │ 地图 Actor 2 (单线程)                      │  │   │
│  │  │ ...                                        │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
│                                                           │
└─────────────────────────────────────────────────────────┘

优势：
✓ 依赖管理清晰：所有依赖在 Dagger2 中定义
✓ 单例共享：避免重复创建，节省内存
✓ 线程安全：Dagger2 编译时生成，Pekko 运行时保证
✓ 易于测试：可以轻松 Mock 依赖
✓ 易于扩展：添加新依赖只需修改 Module
```

---

## 环境搭建步骤

### 1. 项目依赖配置

项目已包含完整的 `pom.xml`，核心依赖版本说明：

```xml
<!-- Pekko 1.0.2：最新稳定版本，推荐用于游戏服 -->
<pekko.version>1.0.2</pekko.version>

<!-- Dagger2 2.51：最新稳定版本，编译时 DI -->
<dagger.version>2.51</dagger.version>

<!-- HikariCP：高性能数据库连接池 -->
<!-- 游戏服推荐配置：20-50 个连接 -->
```

### 2. Maven 编译配置关键点

```xml
<annotationProcessorPaths>
    <path>
        <groupId>com.google.dagger</groupId>
        <artifactId>dagger-compiler</artifactId>
        <version>${dagger.version}</version>
    </path>
</annotationProcessorPaths>
```

**关键：** 必须配置 Dagger2 编译器，否则无法生成依赖注入代码

### 3. 编译和运行

```bash
# 编译项目（会生成 DaggerGameServerComponent）
mvn clean compile

# 运行 Demo
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap"

# 打包成 JAR
mvn clean package
```

---

## 核心实战模块

### 模块 1：GameServerComponent（核心组件定义）

**文件：** `GameServerComponent.java`

**职责：** 定义游戏服的所有依赖和提供方式

```java
@Singleton
@Component(modules = {
    GameServerModule.class,
    ActorSystemModule.class,
    DataAccessModule.class,
    ServiceModule.class
})
public interface GameServerComponent {
    ActorSystem getActorSystem();
    GameServerConfig getGameServerConfig();
    PlayerDAO getPlayerDAO();
    SkillService getSkillService();
    MapService getMapService();
    PlayerActorFactory getPlayerActorFactory();
    MapActorFactory getMapActorFactory();
}
```

**关键设计：**
- `@Singleton` 确保所有依赖只创建一次
- 多个 Module 分别管理不同类型的依赖
- 工厂方法返回已注入依赖的 Actor 工厂

---

### 模块 2：PlayerActorFactory（玩家 Actor 工厂）

**文件：** `PlayerActorFactory.java`

**职责：** 创建玩家 Actor，自动注入所有依赖

```java
public class PlayerActorFactory {
    @Inject
    public PlayerActorFactory(
        ActorSystem actorSystem,
        PlayerDAO playerDAO,
        SkillService skillService
    ) {
        // 此处注入玩家 DAO，避免硬编码依赖
    }
    
    public ActorRef create(long playerId) {
        Props props = Props.create(
            PlayerActor.class,
            playerId,
            playerDAO,      // 已注入的单例
            skillService    // 已注入的单例
        );
        return actorSystem.actorOf(props, "player-" + playerId);
    }
}
```

**使用示例：**

```java
// 获取工厂（已注入所有依赖）
PlayerActorFactory factory = component.getPlayerActorFactory();

// 创建玩家 Actor（无需手动传递依赖）
ActorRef playerActor = factory.create(playerId);

// 发送消息
playerActor.tell(new PlayerMessage.Login(...), ActorRef.noSender());
```

**优势：**
- ✓ 避免 "构造器地狱"
- ✓ 依赖自动注入
- ✓ 所有玩家共享同一个 DAO 和 Service

---

### 模块 3：动态依赖场景（Lazy 和 Provider）

**场景：** 地图 Actor 只在玩家进入时创建，避免启动时创建所有地图

**实现方式：使用 Provider 延迟初始化**

```java
public class MapActorFactory {
    private final Provider<MapService> mapServiceProvider;
    
    @Inject
    public MapActorFactory(
        ActorSystem actorSystem,
        Provider<MapService> mapServiceProvider  // 延迟初始化
    ) {
        this.mapServiceProvider = mapServiceProvider;
    }
    
    public ActorRef create(int mapId) {
        // 只在创建地图 Actor 时才初始化 MapService
        MapService mapService = mapServiceProvider.get();
        
        Props props = Props.create(MapActor.class, mapId, mapService);
        return actorSystem.actorOf(props, "map-" + mapId);
    }
}
```

**对比：**

| 方式 | 初始化时机 | 内存占用 | 适用场景 |
|------|----------|--------|--------|
| `@Singleton` | 启动时 | 高 | 必需的全局资源（ActorSystem、DAO） |
| `Provider<T>` | 首次使用时 | 低 | 可选的动态资源（地图 Actor） |
| `Lazy<T>` | 首次访问时 | 低 | 需要延迟初始化的资源 |

---

## 线程安全方案

### 问题：Dagger2 注入 Pekko Actor 时的线程安全

#### 场景 1：多个线程并发创建 Actor

```java
// ❌ 不安全的做法
ExecutorService executor = Executors.newFixedThreadPool(10);
for (int i = 0; i < 100; i++) {
    final int playerId = i;
    executor.submit(() -> {
        // 多个线程并发调用，可能导致竞态条件
        ActorRef actor = playerActorFactory.create(playerId);
    });
}
```

#### 解决方案 1：ActorSystem 本身是线程安全的

```java
// ✓ 安全的做法
// ActorSystem.actorOf() 是线程安全的
// 内部使用 ConcurrentHashMap 管理 Actor
ExecutorService executor = Executors.newFixedThreadPool(10);
for (int i = 0; i < 100; i++) {
    final int playerId = i;
    executor.submit(() -> {
        // 完全安全，ActorSystem 内部处理并发
        ActorRef actor = playerActorFactory.create(playerId);
        actor.tell(new PlayerMessage.Login(...), ActorRef.noSender());
    });
}
```

**原理：**
- `ActorSystem.actorOf()` 内部使用 `ConcurrentHashMap`
- 每个 Actor 的消息处理是串行的（单线程）
- 不同 Actor 的消息处理可以并行

#### 场景 2：Actor 中访问共享资源

```java
// ❌ 不安全的做法
public class PlayerActor extends AbstractActor {
    private static List<Player> allPlayers = new ArrayList<>();  // 共享可变状态
    
    private void handleLogin(PlayerMessage.Login msg) {
        allPlayers.add(playerData);  // 竞态条件！
    }
}
```

#### 解决方案 2：使用 Actor 管理共享状态

```java
// ✓ 安全的做法
public class PlayerManagerActor extends AbstractActor {
    private final Set<Long> onlinePlayers = new HashSet<>();  // 只在 Actor 中修改
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerOnline.class, msg -> {
                // 此方法在 Actor 的单线程中执行
                onlinePlayers.add(msg.playerId);  // 完全安全
            })
            .build();
    }
}

// 所有线程通过消息通信
playerManagerActor.tell(new PlayerOnline(playerId), ActorRef.noSender());
```

#### 场景 3：DAO 操作的线程安全

```java
// ❌ 不安全的做法
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // 阻塞操作在 Actor 线程中执行，会阻塞其他消息处理
        Player player = playerDAO.findById(playerId);  // 阻塞！
    }
}
```

#### 解决方案 3：使用 blocking-dispatcher

```java
// ✓ 安全的做法
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // 方案 1：使用 pipe 模式
        CompletableFuture.supplyAsync(
            () -> playerDAO.findById(playerId),
            context().dispatcher()  // 使用 blocking-dispatcher
        ).thenAccept(player -> {
            // 回到 Actor 线程处理结果
            self().tell(new LoginResult(player), self());
        });
    }
}
```

或者在 `pom.xml` 中配置：

```xml
<!-- 添加 pekko-actor-typed 支持 @Blocking 注解 -->
<dependency>
    <groupId>org.apache.pekko</groupId>
    <artifactId>pekko-actor-typed_2.13</artifactId>
    <version>${pekko.version}</version>
</dependency>
```

#### 场景 4：Dagger2 生成代码的线程安全

```java
// ✓ 完全安全
// DaggerGameServerComponent 是 Dagger2 编译时生成的类
// 所有依赖关系在编译时已确定，运行时无竞态条件
GameServerComponent component = DaggerGameServerComponent.create();

// 多个线程并发调用，完全安全
ActorSystem system1 = component.getActorSystem();
ActorSystem system2 = component.getActorSystem();
assert system1 == system2;  // 同一个实例
```

**原理：**
- Dagger2 在编译时生成代码
- `@Singleton` 注解确保只创建一次
- 生成的代码使用 `synchronized` 或 `volatile` 保证线程安全

---

## 避坑指南

### 坑 1：Dagger2 编译器未配置

**症状：** 编译时找不到 `DaggerGameServerComponent`

```
error: cannot find symbol
  symbol:   class DaggerGameServerComponent
```

**原因：** 没有配置 Dagger2 注解处理器

**解决方案：**

```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-compiler-plugin</artifactId>
    <configuration>
        <annotationProcessorPaths>
            <path>
                <groupId>com.google.dagger</groupId>
                <artifactId>dagger-compiler</artifactId>
                <version>${dagger.version}</version>
            </path>
        </annotationProcessorPaths>
    </configuration>
</plugin>
```

---

### 坑 2：Actor 类加载冲突

**症状：** 运行时出现 `ClassNotFoundException` 或 `ClassCastException`

**原因：** Dagger2 生成的代码和 Actor 类在不同的 ClassLoader 中

**解决方案：** 确保 Actor 类和 Dagger2 生成的类在同一个 ClassLoader

```java
// ✓ 正确做法
Props props = Props.create(
    PlayerActor.class,  // 使用 Class 对象，而不是实例
    playerId,
    playerDAO,
    skillService
);

// ❌ 错误做法
Props props = Props.create(
    () -> new PlayerActor(playerId, playerDAO, skillService)  // Lambda 可能导致 ClassLoader 问题
);
```

---

### 坑 3：ActorRef 未绑定到正确 Dispatcher

**症状：** Actor 处理消息很慢，CPU 占用率低

**原因：** 数据库操作在默认 Dispatcher 中执行，阻塞了 Actor 线程

**解决方案：** 为阻塞操作使用 blocking-dispatcher

```java
// ❌ 错误做法
private void handleLogin(PlayerMessage.Login msg) {
    Player player = playerDAO.findById(playerId);  // 阻塞！
}

// ✓ 正确做法
private void handleLogin(PlayerMessage.Login msg) {
    // 使用 blocking-dispatcher 执行数据库操作
    CompletableFuture.supplyAsync(
        () -> playerDAO.findById(playerId),
        context().system().dispatchers().lookup("blocking-dispatcher")
    ).thenAccept(player -> {
        self().tell(new LoginResult(player), self());
    });
}
```

---

### 坑 4：Singleton 依赖的初始化顺序

**症状：** 启动时出现 `NullPointerException`

**原因：** 依赖初始化顺序不对，某个依赖在初始化时需要另一个依赖

**解决方案：** 在 Module 中明确指定依赖关系

```java
// ✓ 正确做法
@Module
public class DataAccessModule {
    @Singleton
    @Provides
    static DataSource provideDataSource(GameServerConfig config) {
        // 依赖 GameServerConfig，Dagger2 会自动先初始化它
        return new HikariDataSource(config);
    }
}
```

---

### 坑 5：Provider 的过度使用

**症状：** 每次调用 `provider.get()` 都创建新实例，导致内存泄漏

**原因：** 没有正确理解 Provider 的作用

**解决方案：** 只在需要延迟初始化时使用 Provider

```java
// ❌ 错误做法
@Singleton
public class MapActorFactory {
    private final Provider<MapService> mapServiceProvider;
    
    public ActorRef create(int mapId) {
        // 每次都创建新实例！
        MapService mapService = mapServiceProvider.get();
    }
}

// ✓ 正确做法
@Singleton
public class MapActorFactory {
    private final MapService mapService;  // 直接注入单例
    
    @Inject
    public MapActorFactory(MapService mapService) {
        this.mapService = mapService;
    }
    
    public ActorRef create(int mapId) {
        // 使用同一个实例
        Props props = Props.create(MapActor.class, mapId, mapService);
        return actorSystem.actorOf(props, "map-" + mapId);
    }
}
```

---

### 坑 6：Actor 消息处理中的阻塞

**症状：** 服务器响应变慢，吞吐量下降

**原因：** 在 Actor 的 `receive()` 方法中执行了阻塞操作

**解决方案：** 使用异步模式处理阻塞操作

```java
// ❌ 错误做法
@Override
public Receive createReceive() {
    return receiveBuilder()
        .match(PlayerMessage.Login.class, msg -> {
            // 阻塞操作，会阻塞 Actor 线程
            Player player = playerDAO.findById(playerId);
            sender().tell(new LoginResponse(player), self());
        })
        .build();
}

// ✓ 正确做法
@Override
public Receive createReceive() {
    return receiveBuilder()
        .match(PlayerMessage.Login.class, msg -> {
            // 异步执行，不阻塞 Actor 线程
            CompletableFuture.supplyAsync(
                () -> playerDAO.findById(playerId),
                context().system().dispatchers().lookup("blocking-dispatcher")
            ).thenAccept(player -> {
                sender().tell(new LoginResponse(player), self());
            });
        })
        .build();
}
```

---

### 坑 7：Dagger2 循环依赖

**症状：** 编译时出现 `Cycle in the dependency graph`

**原因：** A 依赖 B，B 依赖 A

**解决方案：** 重构代码，打破循环依赖

```java
// ❌ 错误做法
@Module
public class ModuleA {
    @Provides
    ServiceA provideServiceA(ServiceB serviceB) {
        return new ServiceA(serviceB);
    }
}

@Module
public class ModuleB {
    @Provides
    ServiceB provideServiceB(ServiceA serviceA) {
        return new ServiceB(serviceA);
    }
}

// ✓ 正确做法：使用 Provider 延迟依赖
@Module
public class ModuleA {
    @Provides
    ServiceA provideServiceA(Provider<ServiceB> serviceBProvider) {
        return new ServiceA(serviceBProvider);
    }
}
```

---

## 最佳实践

### 1. 依赖注入的分层

```
GameServerComponent（顶层）
├── GameServerModule（配置）
├── ActorSystemModule（基础设施）
├── DataAccessModule（数据访问）
└── ServiceModule（业务逻辑）
```

**原则：** 上层依赖下层，不能反向依赖

### 2. Actor 的单一职责

```java
// ✓ 好的设计：每个 Actor 只负责一个职责
public class PlayerActor extends AbstractActor {
    // 只负责玩家的状态管理和消息处理
}

public class PlayerManagerActor extends AbstractActor {
    // 只负责所有玩家的统计和管理
}

// ❌ 不好的设计：一个 Actor 负责太多
public class GameServerActor extends AbstractActor {
    // 处理玩家、地图、怪物、副本等所有逻辑
}
```

### 3. 消息设计的不可变性

```java
// ✓ 好的设计：消息是不可变的
public static final class PlayerMessage {
    public final long playerId;
    public final String action;
    
    public PlayerMessage(long playerId, String action) {
        this.playerId = playerId;
        this.action = action;
    }
}

// ❌ 不好的设计：消息是可变的
public class PlayerMessage {
    public long playerId;
    public String action;
    
    public void setPlayerId(long id) {
        this.playerId = id;  // 可能导致竞态条件
    }
}
```

### 4. 错误处理的监督策略

```java
// 在 ActorSystemModule 中配置
Config pekkoConfig = ConfigFactory.parseString(
    "pekko {\n" +
    "  actor {\n" +
    "    guardian-supervisor-strategy = \"com.gameserver.GameServerSupervisor\"\n" +
    "  }\n" +
    "}\n"
);
```

### 5. 日志的统一管理

```java
// ✓ 好的做法：使用 SLF4J + Logback
private static final Logger logger = LoggerFactory.getLogger(PlayerActor.class);

logger.info("[PlayerActor] 玩家登录: playerId={}", playerId);
logger.error("[PlayerActor] 错误: {}", error, exception);

// ❌ 不好的做法：使用 System.out
System.out.println("Player login: " + playerId);
```

---

## 快速开始

### 1. 编译项目

```bash
cd GameServerDagger2Pekko
mvn clean compile
```

### 2. 运行 Demo

```bash
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap"
```

### 3. 预期输出

```
========================================
游戏服务器启动
========================================
[Bootstrap] 初始化 Dagger2 依赖注入容器...
[Bootstrap] Dagger2 容器初始化完成
[Bootstrap] 获取 ActorSystem: GameServer
[Bootstrap] 获取 PlayerActorFactory
[Bootstrap] 获取 MapActorFactory

========================================
演示 1：玩家 Actor 的创建和使用
========================================
[Factory] 创建玩家 Actor: playerId=1001
[PlayerActor] 玩家 Actor 创建: playerId=1001
[Demo] 发送登录消息...
[PlayerActor] 处理登录: playerId=1001, account=player1
...
```

---

## 总结

### 核心要点

1. **Dagger2 的作用**
   - 编译时生成依赖注入代码
   - 确保依赖的单例性
   - 避免 "构造器地狱"

2. **Pekko 的作用**
   - 提供 Actor 并发模型
   - 确保 Actor 的线程安全
   - 支持分布式部署

3. **线程安全保证**
   - Dagger2：编译时生成，无竞态条件
   - Pekko：Actor 单线程，消息队列通信
   - DAO：连接池管理，线程安全

4. **避坑要点**
   - 配置 Dagger2 编译器
   - 使用 blocking-dispatcher 处理阻塞操作
   - 避免 Actor 中的可变共享状态
   - 消息设计要不可变

### 下一步

- 参考 `GameServerBootstrap.java` 了解完整流程
- 查看各个 Module 了解依赖定义
- 根据需求扩展 Actor 和 Service
- 使用 Pekko Cluster 实现分布式部署
