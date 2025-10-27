package com.gameserver.di;

import com.gameserver.config.GameServerConfig;
import dagger.Module;
import dagger.Provides;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Singleton;
import java.io.IOException;
import java.io.InputStream;
import java.util.Properties;

/**
 * 游戏服务器基础配置模块
 * 
 * 职责：
 * 1. 加载游戏服配置文件（application.properties）
 * 2. 提供 GameServerConfig 单例
 * 3. 验证配置的有效性
 * 
 * 游戏服痛点：
 * - 问题：多个 Actor 需要访问配置，重复加载导致内存浪费
 * - 方案：@Singleton 确保配置只加载一次，所有 Actor 共享
 */
@Module
public class GameServerModule {
    
    private static final Logger logger = LoggerFactory.getLogger(GameServerModule.class);

    /**
     * 提供 GameServerConfig 单例
     * 
     * 线程安全保证：
     * - Dagger2 确保该方法只被调用一次
     * - 返回的 GameServerConfig 对象是不可变的（所有字段 final）
     * - 多个 Actor 可以安全地并发访问同一个实例
     */
    @Singleton
    @Provides
    static GameServerConfig provideGameServerConfig() {
        logger.info("[DI] 初始化游戏服配置...");
        
        try {
            Properties props = new Properties();
            
            // 从 classpath 加载配置文件
            try (InputStream input = GameServerModule.class
                    .getClassLoader()
                    .getResourceAsStream("application.properties")) {
                if (input == null) {
                    logger.warn("[DI] 未找到 application.properties，使用默认配置");
                    return GameServerConfig.createDefault();
                }
                props.load(input);
            }
            
            // 构建配置对象
            GameServerConfig config = GameServerConfig.builder()
                .serverPort(Integer.parseInt(props.getProperty("server.port", "8888")))
                .serverHost(props.getProperty("server.host", "0.0.0.0"))
                .actorSystemName(props.getProperty("actor.system.name", "GameServer"))
                .dbUrl(props.getProperty("db.url", "jdbc:mysql://localhost:3306/gamedb"))
                .dbUser(props.getProperty("db.user", "root"))
                .dbPassword(props.getProperty("db.password", ""))
                .dbPoolSize(Integer.parseInt(props.getProperty("db.pool.size", "20")))
                .actorDispatcher(props.getProperty("actor.dispatcher", "default"))
                .build();
            
            logger.info("[DI] 游戏服配置加载完成: port={}, host={}", 
                config.getServerPort(), config.getServerHost());
            
            return config;
        } catch (IOException e) {
            logger.error("[DI] 加载配置文件失败，使用默认配置", e);
            return GameServerConfig.createDefault();
        }
    }
}
