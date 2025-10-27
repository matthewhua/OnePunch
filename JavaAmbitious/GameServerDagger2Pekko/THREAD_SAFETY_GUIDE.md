# Dagger2 + Pekko 线程安全完整指南

## 📌 核心原理

### Dagger2 的线程安全保证

Dagger2 是**编译时**依赖注入框架，所有依赖关系在编译时已确定：

```
编译时：Dagger2 编译器分析依赖关系，生成 DaggerGameServerComponent
↓
运行时：DaggerGameServerComponent 直接执行初始化代码，无需反射
↓
结果：所有依赖关系确定，无竞态条件
```

**关键代码（Dagger2 生成）：**

```java
// 这是 Dagger2 编译时生成的代码（简化版）
public final class DaggerGameServerComponent implements GameServerComponent {
    private final GameServerModule gameServerModule;
    private final ActorSystemModule actorSystemModule;
    
    // 单例缓存
    private ActorSystem actorSystem;
    private GameServerConfig gameServerConfig;
    private PlayerDAO playerDAO;
    
    @Override
    public ActorSystem getActorSystem() {
        if (actorSystem == null) {
            // 第一次调用时初始化
            synchronized (this) {
                if (actorSystem == null) {
                    actorSystem = ActorSystemModule.provideActorSystem(getGameServerConfig());
                }
            }
        }
        return actorSystem;
    }
    
    // 其他 getter 方法类似
}
```

**线程安全机制：**
- ✓ 使用 `synchronized` 块保护单例初始化
- ✓ 双重检查锁定（Double-Checked Locking）
- ✓ 编译时已确定所有依赖关系

---

## 🎯 场景 1：多线程并发创建 Actor

### 问题描述

```java
// 多个线程并发创建玩家 Actor
ExecutorService executor = Executors.newFixedThreadPool(10);
for (int i = 0; i < 100; i++) {
    final int playerId = i;
    executor.submit(() -> {
        // 多个线程并发调用，是否线程安全？
        ActorRef actor = playerActorFactory.create(playerId);
        actor.tell(new PlayerMessage.Login(...), ActorRef.noSender());
    });
}
```

### 分析

| 操作 | 线程安全性 | 原因 |
|------|----------|------|
| `playerActorFactory.create()` | ✓ 安全 | ActorSystem.actorOf() 是线程安全的 |
| `actor.tell()` | ✓ 安全 | ActorRef 是不可变的，消息队列是线程安全的 |
| 整个流程 | ✓ 安全 | Pekko 框架内部处理所有并发 |

### 实现原理

```java
// ActorSystem.actorOf() 的内部实现（简化版）
public ActorRef actorOf(Props props, String name) {
    // 使用 ConcurrentHashMap 管理 Actor
    ActorRef actorRef = new ActorRefImpl(props, name);
    
    // 线程安全的 Actor 注册
    synchronized (this.actors) {
        this.actors.put(name, actorRef);
    }
    
    return actorRef;
}

// ActorRef.tell() 的内部实现（简化版）
public void tell(Object message, ActorRef sender) {
    // 消息加入队列（线程安全的阻塞队列）
    this.mailbox.enqueue(new Message(message, sender));
    
    // 如果 Actor 线程空闲，唤醒它
    this.dispatcher.schedule(this::processMessages);
}
```

### 最佳实践

```java
// ✓ 正确做法：多线程并发创建 Actor
public class GameServer {
    private final PlayerActorFactory playerActorFactory;
    private final ExecutorService executor;
    
    public void handlePlayerLogin(long playerId, String account) {
        // 在线程池中异步创建 Actor
        executor.submit(() -> {
            ActorRef playerActor = playerActorFactory.create(playerId);
            playerActor.tell(
                new PlayerMessage.Login(account, "password"),
                ActorRef.noSender()
            );
        });
    }
}
```

---

## 🎯 场景 2：Actor 中的共享状态管理

### 问题描述

```java
// ❌ 不安全的做法
public class PlayerManagerActor extends AbstractActor {
    private List<Long> onlinePlayers = new ArrayList<>();  // 共享可变状态
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerOnline.class, msg -> {
                // 多个线程可能并发访问 onlinePlayers
                onlinePlayers.add(msg.playerId);  // 竞态条件！
            })
            .build();
    }
}
```

### 分析

**问题根源：** 虽然 Actor 本身是单线程的，但如果多个线程直接访问 Actor 的字段，就会产生竞态条件。

### 解决方案

```java
// ✓ 正确做法 1：只通过消息通信
public class PlayerManagerActor extends AbstractActor {
    private final Set<Long> onlinePlayers = new HashSet<>();
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerOnline.class, msg -> {
                // 此方法在 Actor 的单线程中执行
                // onlinePlayers 只在这里修改，完全安全
                onlinePlayers.add(msg.playerId);
            })
            .match(GetOnlineCount.class, msg -> {
                // 返回当前在线人数
                sender().tell(new OnlineCount(onlinePlayers.size()), self());
            })
            .build();
    }
}

// 所有线程通过消息通信
playerManagerActor.tell(new PlayerOnline(playerId), ActorRef.noSender());
```

**原理：**
- ✓ Actor 的 `receive()` 方法在 Actor 的单线程中执行
- ✓ 共享状态只在 `receive()` 方法中修改
- ✓ 不同线程通过消息队列通信，无竞态条件

```java
// ✓ 正确做法 2：使用不可变数据结构
public class PlayerManagerActor extends AbstractActor {
    private Set<Long> onlinePlayers = Collections.emptySet();
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerOnline.class, msg -> {
                // 创建新的不可变集合
                Set<Long> newSet = new HashSet<>(onlinePlayers);
                newSet.add(msg.playerId);
                onlinePlayers = Collections.unmodifiableSet(newSet);
            })
            .build();
    }
}
```

---

## 🎯 场景 3：DAO 操作的线程安全

### 问题描述

```java
// ❌ 不安全的做法
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // 阻塞操作在 Actor 线程中执行
        // 会阻塞其他消息处理，导致吞吐量下降
        Player player = playerDAO.findById(playerId);  // 阻塞！
        
        sender().tell(new LoginResponse(player), self());
    }
}
```

### 分析

**问题：**
1. 数据库查询可能耗时 10-100ms
2. 在这段时间内，Actor 无法处理其他消息
3. 其他玩家的消息被堵在队列中，导致延迟增加

### 解决方案 1：使用 blocking-dispatcher

```java
// ✓ 正确做法 1：使用 blocking-dispatcher
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // 获取 blocking-dispatcher
        ExecutionContext blockingDispatcher = 
            context().system().dispatchers().lookup("blocking-dispatcher");
        
        // 在 blocking-dispatcher 中执行数据库操作
        CompletableFuture.supplyAsync(
            () -> playerDAO.findById(playerId),
            blockingDispatcher
        ).thenAccept(player -> {
            // 回到 Actor 线程处理结果
            self().tell(new LoginResult(player), self());
        });
    }
}
```

**原理：**
- ✓ `blocking-dispatcher` 是一个独立的线程池
- ✓ 数据库操作在这个线程池中执行，不阻塞 Actor 线程
- ✓ 结果通过消息返回给 Actor，保证线程安全

### 解决方案 2：使用 ask 模式

```java
// ✓ 正确做法 2：使用 ask 模式
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // 创建一个临时 Actor 处理数据库操作
        ActorRef dbActor = context().actorOf(Props.create(
            DatabaseActor.class, playerDAO
        ));
        
        // 使用 ask 模式等待结果
        Patterns.ask(dbActor, new QueryPlayer(playerId), Duration.ofSeconds(5))
            .thenAccept(result -> {
                sender().tell(new LoginResponse((Player) result), self());
            });
    }
}

// DatabaseActor 在独立线程中执行数据库操作
public class DatabaseActor extends AbstractActor {
    private final PlayerDAO playerDAO;
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(QueryPlayer.class, msg -> {
                Player player = playerDAO.findById(msg.playerId);
                sender().tell(player, self());
            })
            .build();
    }
}
```

### 解决方案 3：使用 Pekko Persistence

```java
// ✓ 正确做法 3：使用 Pekko Persistence
public class PlayerActor extends AbstractActor {
    private void handleLogin(PlayerMessage.Login msg) {
        // Pekko Persistence 自动处理异步数据库操作
        persist(new PlayerLoggedIn(playerId), event -> {
            // 事件已持久化，更新状态
            playerData = loadPlayerData(playerId);
            sender().tell(new LoginResponse(playerData), self());
        });
    }
}
```

---

## 🎯 场景 4：Dagger2 容器的并发访问

### 问题描述

```java
// 多个线程并发获取依赖
ExecutorService executor = Executors.newFixedThreadPool(10);
GameServerComponent component = DaggerGameServerComponent.create();

for (int i = 0; i < 100; i++) {
    executor.submit(() -> {
        // 多个线程并发调用，是否线程安全？
        ActorSystem system = component.getActorSystem();
        PlayerDAO dao = component.getPlayerDAO();
    });
}
```

### 分析

| 操作 | 线程安全性 | 原因 |
|------|----------|------|
| `component.getActorSystem()` | ✓ 安全 | Dagger2 使用 synchronized 保护单例 |
| `component.getPlayerDAO()` | ✓ 安全 | Dagger2 使用 synchronized 保护单例 |
| 整个流程 | ✓ 安全 | 所有单例初始化都是线程安全的 |

### 实现原理

```java
// Dagger2 生成的代码（简化版）
public final class DaggerGameServerComponent implements GameServerComponent {
    private volatile ActorSystem actorSystem;
    private volatile PlayerDAO playerDAO;
    
    @Override
    public ActorSystem getActorSystem() {
        if (actorSystem == null) {
            synchronized (this) {
                if (actorSystem == null) {
                    // 初始化 ActorSystem
                    actorSystem = ActorSystemModule.provideActorSystem(...);
                }
            }
        }
        return actorSystem;
    }
    
    @Override
    public PlayerDAO getPlayerDAO() {
        if (playerDAO == null) {
            synchronized (this) {
                if (playerDAO == null) {
                    // 初始化 PlayerDAO
                    playerDAO = DataAccessModule.providePlayerDAO(...);
                }
            }
        }
        return playerDAO;
    }
}
```

**关键点：**
- ✓ 使用 `volatile` 关键字保证可见性
- ✓ 使用 `synchronized` 块保证原子性
- ✓ 双重检查锁定避免不必要的同步

---

## 🎯 场景 5：数据库连接池的线程安全

### 问题描述

```java
// HikariCP 连接池是否线程安全？
DataSource dataSource = new HikariDataSource(config);

// 多个线程并发获取连接
ExecutorService executor = Executors.newFixedThreadPool(20);
for (int i = 0; i < 100; i++) {
    executor.submit(() -> {
        try (Connection conn = dataSource.getConnection()) {
            // 执行数据库操作
        }
    });
}
```

### 分析

**HikariCP 的线程安全保证：**

```
┌─────────────────────────────────────────┐
│  HikariCP 连接池（线程安全）             │
├─────────────────────────────────────────┤
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ ConcurrentBag（无锁队列）       │   │
│  │ - 存储可用连接                  │   │
│  │ - 线程安全的 get/put 操作       │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ 连接验证（自动）                │   │
│  │ - 连接泄漏检测                  │   │
│  │ - 连接超时管理                  │   │
│  └─────────────────────────────────┘   │
│                                         │
└─────────────────────────────────────────┘
```

**线程安全机制：**
- ✓ 使用 `ConcurrentBag` 存储连接（无锁数据结构）
- ✓ 每个线程获取独立的连接
- ✓ 连接使用完毕后自动归还

### 最佳实践

```java
// ✓ 正确配置 HikariCP
HikariConfig config = new HikariConfig();
config.setJdbcUrl("jdbc:mysql://localhost:3306/gamedb");
config.setUsername("root");
config.setPassword("");

// 连接池大小配置
config.setMaximumPoolSize(20);      // 最大连接数
config.setMinimumIdle(5);           // 最小空闲连接数
config.setConnectionTimeout(30000); // 获取连接超时

// 连接验证
config.setConnectionTestQuery("SELECT 1");
config.setLeakDetectionThreshold(60000);  // 60 秒检测泄漏

HikariDataSource dataSource = new HikariDataSource(config);
```

---

## 🎯 场景 6：Actor 消息的线程安全

### 问题描述

```java
// 消息是否需要是不可变的？
public class PlayerMessage {
    public long playerId;
    public String action;
    
    public void setPlayerId(long id) {
        this.playerId = id;  // 可变消息
    }
}

// 发送消息
playerActor.tell(msg, sender);
```

### 分析

**问题：** 如果消息是可变的，发送者可能在 Actor 处理消息时修改消息内容，导致竞态条件。

### 解决方案

```java
// ✓ 正确做法：消息必须是不可变的
public static final class PlayerMessage {
    public final long playerId;
    public final String action;
    
    public PlayerMessage(long playerId, String action) {
        this.playerId = playerId;
        this.action = action;
    }
    
    // 没有 setter 方法
}

// 或者使用 record（Java 16+）
public record PlayerMessage(long playerId, String action) {}
```

**原理：**
- ✓ 不可变消息无法被修改
- ✓ 多个线程可以安全地共享同一个消息对象
- ✓ 避免了消息在传输过程中被修改

---

## 📊 线程安全总结表

| 组件 | 线程安全性 | 机制 | 使用场景 |
|------|----------|------|--------|
| Dagger2 容器 | ✓ 安全 | synchronized + volatile | 依赖注入 |
| ActorSystem | ✓ 安全 | ConcurrentHashMap | Actor 创建 |
| ActorRef | ✓ 安全 | 不可变对象 | 消息发送 |
| Actor 消息队列 | ✓ 安全 | 阻塞队列 | 消息处理 |
| HikariCP 连接池 | ✓ 安全 | ConcurrentBag | 数据库操作 |
| Actor 内部状态 | ✓ 安全 | 单线程执行 | 状态管理 |
| 共享可变状态 | ✗ 不安全 | 需要同步 | 避免使用 |

---

## 🔍 调试线程安全问题

### 工具 1：JVM 参数

```bash
# 启用线程安全检查
java -XX:+UnlockDiagnosticVMOptions \
     -XX:+PrintCompilation \
     -XX:+LogCompilation \
     -XX:LogFile=compilation.log \
     -jar gameserver.jar
```

### 工具 2：Pekko 调试

```xml
<!-- pom.xml -->
<dependency>
    <groupId>org.apache.pekko</groupId>
    <artifactId>pekko-slf4j_2.13</artifactId>
    <version>${pekko.version}</version>
</dependency>
```

```properties
# application.properties
pekko.loglevel = DEBUG
pekko.actor.debug.receive = on
pekko.actor.debug.autoreceive = on
```

### 工具 3：ThreadSanitizer（Linux）

```bash
# 编译时启用 ThreadSanitizer
gcc -fsanitize=thread -g -o gameserver gameserver.c
```

---

## ✅ 线程安全检查清单

- [ ] Dagger2 编译器已配置
- [ ] 所有 @Singleton 依赖都是线程安全的
- [ ] Actor 消息都是不可变的
- [ ] 数据库操作使用 blocking-dispatcher
- [ ] 共享状态只在 Actor 中修改
- [ ] 没有在 Actor 外直接访问 Actor 的字段
- [ ] HikariCP 连接池已正确配置
- [ ] 所有线程通过消息队列通信
- [ ] 没有使用 `synchronized` 块（除非必要）
- [ ] 没有使用 `volatile` 字段（除非必要）

---

## 📚 参考资源

- [Pekko 官方文档](https://pekko.apache.org/)
- [Dagger 官方文档](https://dagger.dev/)
- [HikariCP 文档](https://github.com/brettwooldridge/HikariCP)
- [Java 并发编程](https://docs.oracle.com/javase/tutorial/essential/concurrency/)
