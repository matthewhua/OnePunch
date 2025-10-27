# 快速开始指南

## 5 分钟快速上手

### 第一步：编译项目

```bash
cd GameServerDagger2Pekko
mvn clean compile
```

**预期输出：**
```
[INFO] BUILD SUCCESS
```

**如果出现错误：**
```
error: cannot find symbol
  symbol:   class DaggerGameServerComponent
```

**解决方案：** 确保 `pom.xml` 中配置了 Dagger2 编译器

```xml
<annotationProcessorPaths>
    <path>
        <groupId>com.google.dagger</groupId>
        <artifactId>dagger-compiler</artifactId>
        <version>${dagger.version}</version>
    </path>
</annotationProcessorPaths>
```

---

### 第二步：运行 Demo

```bash
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap"
```

**预期输出：**
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

### 第三步：运行单元测试

```bash
mvn test
```

**预期输出：**
```
[INFO] Tests run: 10, Failures: 0, Errors: 0, Skipped: 0
[INFO] BUILD SUCCESS
```

---

## 核心概念速览

### 1. Dagger2 依赖注入

**问题：** 如何管理游戏服的所有依赖？

```java
// ❌ 传统做法：手动创建所有依赖
ActorSystem system = ActorSystem.create("GameServer");
DataSource dataSource = new HikariDataSource(config);
PlayerDAO playerDAO = new PlayerDAOImpl(dataSource);
SkillService skillService = new SkillServiceImpl();
PlayerActorFactory factory = new PlayerActorFactory(system, playerDAO, skillService);

// ✓ Dagger2 做法：自动注入
GameServerComponent component = DaggerGameServerComponent.create();
PlayerActorFactory factory = component.getPlayerActorFactory();
```

**优势：**
- ✓ 依赖关系清晰
- ✓ 单例自动管理
- ✓ 易于测试

### 2. Pekko Actor 模型

**问题：** 如何处理并发的玩家请求？

```java
// ✓ 每个玩家对应一个 Actor
ActorRef playerActor = playerActorFactory.create(playerId);

// 发送消息（线程安全）
playerActor.tell(new PlayerMessage.Login(...), ActorRef.noSender());

// Actor 在自己的线程中处理消息
// 不同 Actor 可以并行处理，避免竞态条件
```

**优势：**
- ✓ 天然支持并发
- ✓ 无需手动加锁
- ✓ 消息驱动，易于理解

### 3. 线程安全保证

| 层级 | 机制 | 结果 |
|------|------|------|
| Dagger2 | 编译时生成代码 | 依赖注入无竞态条件 |
| Pekko | Actor 单线程 | 消息处理无竞态条件 |
| 数据库 | 连接池管理 | 数据库访问无竞态条件 |

---

## 常见问题

### Q1：如何添加新的依赖？

**步骤 1：** 创建 Service 接口和实现

```java
public interface MyService {
    void doSomething();
}

public class MyServiceImpl implements MyService {
    @Override
    public void doSomething() {
        // 实现逻辑
    }
}
```

**步骤 2：** 在 Module 中提供依赖

```java
@Module
public class MyModule {
    @Singleton
    @Provides
    static MyService provideMyService() {
        return new MyServiceImpl();
    }
}
```

**步骤 3：** 在 Component 中添加 Module

```java
@Component(modules = {
    GameServerModule.class,
    ActorSystemModule.class,
    DataAccessModule.class,
    ServiceModule.class,
    MyModule.class  // 新增
})
public interface GameServerComponent {
    MyService getMyService();
}
```

**步骤 4：** 重新编译

```bash
mvn clean compile
```

---

### Q2：如何在 Actor 中使用注入的依赖？

```java
// 步骤 1：在 Actor 工厂中注入依赖
public class MyActorFactory {
    private final MyService myService;
    
    @Inject
    public MyActorFactory(MyService myService) {
        this.myService = myService;
    }
    
    public ActorRef create() {
        Props props = Props.create(
            MyActor.class,
            myService  // 传递给 Actor
        );
        return actorSystem.actorOf(props, "my-actor");
    }
}

// 步骤 2：在 Actor 中使用依赖
public class MyActor extends AbstractActor {
    private final MyService myService;
    
    public MyActor(MyService myService) {
        this.myService = myService;
    }
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(MyMessage.class, msg -> {
                myService.doSomething();
            })
            .build();
    }
}
```

---

### Q3：如何处理数据库操作的阻塞？

```java
// 使用 blocking-dispatcher
private void handleDatabaseOperation(Message msg) {
    CompletableFuture.supplyAsync(
        () -> playerDAO.findById(playerId),
        context().system().dispatchers().lookup("blocking-dispatcher")
    ).thenAccept(player -> {
        self().tell(new Result(player), self());
    });
}
```

---

### Q4：如何调试 Actor 的消息处理？

```properties
# application.properties
pekko.loglevel = DEBUG
pekko.actor.debug.receive = on
pekko.actor.debug.autoreceive = on
```

**输出示例：**
```
[DEBUG] PlayerActor received message: Login{account='player1'}
[DEBUG] PlayerActor sent message: LoginResponse{success=true}
```

---

### Q5：如何测试 Actor？

```java
@Test
public void testPlayerActorLogin() {
    new TestKit(system) {
        {
            // 创建 Mock 依赖
            PlayerDAO playerDAO = mock(PlayerDAO.class);
            when(playerDAO.findById(1001))
                .thenReturn(Optional.of(mockPlayer));
            
            // 创建 Actor
            ActorRef playerActor = system.actorOf(
                Props.create(PlayerActor.class, 1001L, playerDAO, skillService),
                "test-player"
            );
            
            // 发送消息
            playerActor.tell(new PlayerMessage.Login(...), getRef());
            
            // 验证响应
            PlayerMessage.LoginResponse response = expectMsgClass(
                duration("1 second"),
                PlayerMessage.LoginResponse.class
            );
            
            assert response.success;
        }
    };
}
```

---

## 项目结构

```
GameServerDagger2Pekko/
├── pom.xml                          # Maven 配置
├── README.md                        # 详细教程
├── QUICKSTART.md                    # 本文件
├── THREAD_SAFETY_GUIDE.md          # 线程安全指南
│
├── src/main/java/com/gameserver/
│   ├── GameServerBootstrap.java     # 启动类
│   │
│   ├── di/                          # 依赖注入
│   │   ├── GameServerComponent.java # 核心组件
│   │   ├── GameServerModule.java    # 配置模块
│   │   ├── ActorSystemModule.java   # ActorSystem 模块
│   │   ├── DataAccessModule.java    # 数据访问模块
│   │   └── ServiceModule.java       # 服务模块
│   │
│   ├── config/                      # 配置
│   │   └── GameServerConfig.java    # 服务器配置
│   │
│   ├── model/                       # 数据模型
│   │   └── Player.java              # 玩家模型
│   │
│   ├── dao/                         # 数据访问层
│   │   ├── PlayerDAO.java           # 玩家 DAO 接口
│   │   └── impl/
│   │       └── PlayerDAOImpl.java    # 玩家 DAO 实现
│   │
│   ├── service/                     # 业务服务层
│   │   ├── SkillService.java        # 技能服务接口
│   │   ├── MapService.java          # 地图服务接口
│   │   └── impl/
│   │       ├── SkillServiceImpl.java # 技能服务实现
│   │       └── MapServiceImpl.java   # 地图服务实现
│   │
│   └── actor/                       # Actor 层
│       ├── PlayerActorFactory.java  # 玩家 Actor 工厂
│       ├── PlayerActor.java         # 玩家 Actor
│       ├── PlayerMessage.java       # 玩家消息
│       ├── MapActorFactory.java     # 地图 Actor 工厂
│       ├── MapActor.java            # 地图 Actor
│       └── MapMessage.java          # 地图消息
│
├── src/main/resources/
│   ├── application.properties       # 应用配置
│   └── logback.xml                  # 日志配置
│
└── src/test/java/com/gameserver/
    ├── di/
    │   └── GameServerComponentTest.java    # 组件测试
    └── actor/
        └── PlayerActorTest.java            # Actor 测试
```

---

## 下一步

1. **学习 README.md** - 了解完整的技术细节
2. **学习 THREAD_SAFETY_GUIDE.md** - 深入理解线程安全
3. **修改 Demo** - 根据需求扩展功能
4. **添加新 Actor** - 实现地图、副本等功能
5. **部署到生产** - 使用 Pekko Cluster 实现分布式部署

---

## 获取帮助

### 常见错误

| 错误 | 原因 | 解决方案 |
|------|------|--------|
| `ClassNotFoundException: DaggerGameServerComponent` | Dagger2 编译器未配置 | 检查 pom.xml 中的 annotationProcessorPaths |
| `NullPointerException` | 依赖未初始化 | 确保在 Module 中提供了依赖 |
| `ActorNotFound` | Actor 名称不正确 | 检查 actorOf() 中的 Actor 名称 |
| `TimeoutException` | Actor 响应超时 | 检查 Actor 是否正确处理消息 |

### 调试技巧

```bash
# 启用详细日志
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap" \
    -Dlogback.configurationFile=src/main/resources/logback.xml

# 查看 Dagger2 生成的代码
find target -name "Dagger*.java" | head -5

# 运行单个测试
mvn test -Dtest=GameServerComponentTest#testActorSystemSingleton
```

---

## 总结

✓ 5 分钟快速上手  
✓ 完整的 Demo 代码  
✓ 详细的文档和注释  
✓ 单元测试覆盖  
✓ 线程安全保证  

**现在你已经准备好开发游戏服务器了！**
