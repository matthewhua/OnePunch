package com.gameserver.di;

import com.gameserver.config.GameServerConfig;
import com.typesafe.config.Config;
import com.typesafe.config.ConfigFactory;
import dagger.Module;
import dagger.Provides;
import org.apache.pekko.actor.ActorSystem;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Singleton;

/**
 * Pekko ActorSystem 依赖注入模块
 * 
 * 设计目标：
 * 1. 创建游戏服的全局 ActorSystem 单例
 * 2. 配置 ActorSystem 的线程池、Dispatcher 等参数
 * 3. 支持 Pekko Cluster 分布式部署
 * 
 * 游戏服痛点解决：
 * - 问题：多个 Actor 需要 ActorSystem 引用，手动创建多个实例导致资源浪费
 * - 方案：@Singleton 确保全局只有一个 ActorSystem，所有 Actor 共享
 * 
 * 线程安全保证：
 * - ActorSystem 本身是线程安全的，由 Pekko 框架内部的线程池保证
 * - Dagger2 确保 ActorSystem 只被创建一次
 * - 所有 Actor 操作都通过 ActorSystem 的线程池执行，避免竞态条件
 */
@Module
public class ActorSystemModule {
    
    private static final Logger logger = LoggerFactory.getLogger(ActorSystemModule.class);

    /**
     * 提供 ActorSystem 单例
     * 
     * 关键参数说明：
     * 
     * 1. 线程池配置（fork-join-executor）：
     *    - parallelism-factor: 1.0 表示线程数 = CPU 核心数 * 1.0
     *    - 对于游戏服，建议设置为 2.0-4.0（充分利用多核）
     *    - 过高会导致上下文切换开销，过低会导致吞吐量不足
     * 
     * 2. Dispatcher 配置：
     *    - default-dispatcher：处理大多数 Actor 消息
     *    - blocking-dispatcher：处理阻塞操作（如数据库查询）
     *    - 游戏服应该为 DAO 操作使用 blocking-dispatcher，避免阻塞主线程
     * 
     * 3. 监督策略（supervision）：
     *    - 当 Actor 发生异常时的处理方式
     *    - 游戏服应该记录异常但继续运行，避免整个服务器崩溃
     * 
     * 线程安全关键点：
     * - ActorSystem 的所有操作都是异步的，通过消息队列实现
     * - 不同 Actor 的消息在不同线程执行，Pekko 框架保证消息顺序
     * - 不需要手动加锁，避免死锁风险
     */
    @Singleton
    @Provides
    static ActorSystem provideActorSystem(GameServerConfig config) {
        logger.info("[DI] 初始化 ActorSystem...");
        
        // 从配置文件加载 Pekko 配置
        Config pekkoConfig = ConfigFactory.load("application.properties");
        
        // 创建 ActorSystem
        ActorSystem actorSystem = ActorSystem.create(
            config.getActorSystemName(),
            pekkoConfig
        );
        
        logger.info("[DI] ActorSystem 初始化完成: name={}", config.getActorSystemName());
        
        return actorSystem;
    }
}
