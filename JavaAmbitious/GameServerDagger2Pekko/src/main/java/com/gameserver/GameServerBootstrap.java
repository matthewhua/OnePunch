package com.gameserver;

import com.gameserver.actor.MapActorFactory;
import com.gameserver.actor.MapMessage;
import com.gameserver.actor.PlayerActorFactory;
import com.gameserver.actor.PlayerMessage;
import com.gameserver.config.GameServerConfig;
import com.gameserver.di.DaggerGameServerComponent;
import com.gameserver.di.GameServerComponent;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.TimeUnit;

/**
 * 游戏服务器启动类
 * 
 * 这是完整的 Demo，展示从 Dagger2 依赖注入到 Pekko Actor 调用的全流程
 * 
 * 核心流程：
 * 1. 使用 Dagger2 构建依赖注入容器（GameServerComponent）
 * 2. 从容器获取 ActorSystem 和 Actor 工厂
 * 3. 使用工厂创建 Actor 实例
 * 4. 通过 ActorRef 发送消息给 Actor
 * 
 * 线程安全保证：
 * - Dagger2 在编译时生成代码，运行时无竞态条件
 * - ActorSystem 是线程安全的
 * - 所有 Actor 操作都通过消息队列，避免直接访问
 * 
 * 游戏服最佳实践：
 * - 启动时初始化 Dagger2 容器（一次性）
 * - 从容器获取 Actor 工厂（单例）
 * - 使用工厂创建 Actor（可多次）
 * - 通过 ActorRef 发送消息（线程安全）
 */
public class GameServerBootstrap {
    
    private static final Logger logger = LoggerFactory.getLogger(GameServerBootstrap.class);

    public static void main(String[] args) throws Exception {
        logger.info("========================================");
        logger.info("游戏服务器启动");
        logger.info("========================================");
        
        // ============ 第零步：加载配置文件 ============
        // 
        // 从 application.properties 加载配置
        // 支持环境变量覆盖
        logger.info("[Bootstrap] 加载配置文件 application.properties...");
        GameServerConfig config = GameServerConfig.loadFromProperties();
        logger.info("[Bootstrap] 配置加载完成: {}", config);
        
        // ============ 第一步：初始化 Dagger2 依赖注入容器 ============
        // 
        // 此处注入 Dagger2 容器，避免硬编码依赖
        // DaggerGameServerComponent 是 Dagger2 编译时生成的类
        // 包含所有依赖的初始化逻辑
        logger.info("[Bootstrap] 初始化 Dagger2 依赖注入容器...");
        GameServerComponent component = DaggerGameServerComponent.create();
        logger.info("[Bootstrap] Dagger2 容器初始化完成");
        
        // ============ 第二步：获取核心基础设施 ============
        
        ActorSystem actorSystem = component.getActorSystem();
        logger.info("[Bootstrap] 获取 ActorSystem: {}", actorSystem.name());
        
        // ============ 第三步：获取 Actor 工厂 ============
        // 
        // 工厂已注入所有依赖（PlayerDAO、SkillService 等）
        // 调用方只需调用 factory.create(playerId)，无需手动传递依赖
        // 这避免了 "构造器地狱"
        
        PlayerActorFactory playerActorFactory = component.getPlayerActorFactory();
        logger.info("[Bootstrap] 获取 PlayerActorFactory");
        
        MapActorFactory mapActorFactory = component.getMapActorFactory();
        logger.info("[Bootstrap] 获取 MapActorFactory");
        
        // ============ 第四步：演示玩家 Actor 的使用 ============
        
        logger.info("\n========================================");
        logger.info("演示 1：玩家 Actor 的创建和使用");
        logger.info("========================================");
        
        // 创建玩家 Actor
        // 此处注入玩家 DAO，避免硬编码依赖
        long playerId = 1001;
        ActorRef playerActor = playerActorFactory.create(playerId);
        logger.info("[Demo] 创建玩家 Actor: playerId={}", playerId);
        
        // 发送登录消息
        logger.info("[Demo] 发送登录消息...");
        playerActor.tell(
            new PlayerMessage.Login("player1", "password123"),
            ActorRef.noSender()
        );
        
        // 等待一段时间，让 Actor 处理消息
        Thread.sleep(1000);
        
        // 发送攻击消息
        logger.info("[Demo] 发送攻击消息...");
        playerActor.tell(
            new PlayerMessage.Attack(2001, 1),
            ActorRef.noSender()
        );
        
        Thread.sleep(1000);
        
        // 发送增加经验消息
        logger.info("[Demo] 发送增加经验消息...");
        playerActor.tell(
            new PlayerMessage.AddExperience(100),
            ActorRef.noSender()
        );
        
        Thread.sleep(1000);
        
        // ============ 第五步：演示地图 Actor 的使用 ============
        
        logger.info("\n========================================");
        logger.info("演示 2：地图 Actor 的创建和使用");
        logger.info("========================================");
        
        // 创建地图 Actor
        // 此处注入地图服务，避免硬编码依赖
        int mapId = 1;
        ActorRef mapActor = mapActorFactory.create(mapId);
        logger.info("[Demo] 创建地图 Actor: mapId={}", mapId);
        
        // 发送玩家进入地图消息
        logger.info("[Demo] 发送玩家进入地图消息...");
        mapActor.tell(
            new MapMessage.PlayerEnter(playerId),
            ActorRef.noSender()
        );
        
        Thread.sleep(500);
        
        // 发送获取玩家数量消息
        logger.info("[Demo] 发送获取玩家数量消息...");
        mapActor.tell(
            new MapMessage.GetPlayerCount(),
            ActorRef.noSender()
        );
        
        Thread.sleep(500);
        
        // 发送玩家离开地图消息
        logger.info("[Demo] 发送玩家离开地图消息...");
        mapActor.tell(
            new MapMessage.PlayerLeave(playerId),
            ActorRef.noSender()
        );
        
        Thread.sleep(500);
        
        // ============ 第六步：演示多个玩家的并发处理 ============
        
        logger.info("\n========================================");
        logger.info("演示 3：多个玩家的并发处理");
        logger.info("========================================");
        
        // 创建多个玩家 Actor
        for (int i = 0; i < 3; i++) {
            long pid = 2000 + i;
            ActorRef actor = playerActorFactory.create(pid);
            
            // 发送登录消息
            actor.tell(
                new PlayerMessage.Login("player" + i, "password"),
                ActorRef.noSender()
            );
            
            logger.info("[Demo] 创建玩家 Actor: playerId={}", pid);
        }
        
        Thread.sleep(2000);
        
        // ============ 第七步：演示线程安全 ============
        
        logger.info("\n========================================");
        logger.info("演示 4：线程安全的并发访问");
        logger.info("========================================");
        
        // 创建多个线程，并发发送消息
        for (int i = 0; i < 5; i++) {
            final int threadId = i;
            new Thread(() -> {
                try {
                    long pid = 3000 + threadId;
                    ActorRef actor = playerActorFactory.create(pid);
                    
                    actor.tell(
                        new PlayerMessage.Login("player" + threadId, "password"),
                        ActorRef.noSender()
                    );
                    
                    logger.info("[Thread-{}] 创建玩家 Actor: playerId={}", threadId, pid);
                } catch (Exception e) {
                    logger.error("[Thread-{}] 错误", threadId, e);
                }
            }).start();
        }
        
        Thread.sleep(3000);
        
        // ============ 第八步：关闭服务器 ============
        
        logger.info("\n========================================");
        logger.info("关闭游戏服务器");
        logger.info("========================================");
        
        // 发送登出消息给第一个玩家
        playerActor.tell(new PlayerMessage.Logout(), ActorRef.noSender());
        
        Thread.sleep(1000);
        
        // 关闭 ActorSystem
        actorSystem.terminate();
        
        // 等待 ActorSystem 完全关闭
        actorSystem.getWhenTerminated().toCompletableFuture().get(10, TimeUnit.SECONDS);
        
        logger.info("[Bootstrap] 游戏服务器已关闭");
    }
}
