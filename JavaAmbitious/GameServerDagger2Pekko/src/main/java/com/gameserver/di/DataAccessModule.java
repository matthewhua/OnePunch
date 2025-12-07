package com.gameserver.di;

import com.gameserver.config.GameServerConfig;
import com.gameserver.dao.PlayerDAO;
import com.gameserver.dao.impl.PlayerDAOImpl;
import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import dagger.Module;
import dagger.Provides;
import org.hibernate.SessionFactory;
import org.hibernate.boot.MetadataSources;
import org.hibernate.boot.registry.StandardServiceRegistry;
import org.hibernate.boot.registry.StandardServiceRegistryBuilder;
import org.hibernate.cfg.AvailableSettings;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Singleton;
import javax.sql.DataSource;
import java.util.HashMap;
import java.util.Map;

/**
 * 数据访问层依赖注入模块
 * 
 * 设计目标：
 * 1. 创建数据库连接池（HikariCP）单例
 * 2. 提供 DAO 实现类的依赖注入
 * 3. 确保数据库连接的线程安全和高效利用
 * 
 * 游戏服痛点解决：
 * - 问题：多个玩家 Actor 需要访问数据库，每个 Actor 创建连接导致资源浪费
 * - 方案：使用连接池，所有 Actor 共享有限的连接，提高吞吐量
 * 
 * 线程安全保证：
 * - HikariCP 连接池本身是线程安全的
 * - 每个 Actor 从池中获取连接，使用完毕后归还
 * - 建议在 blocking-dispatcher 中执行数据库操作，避免阻塞 Actor 线程
 */
@Module
public class DataAccessModule {
    
    private static final Logger logger = LoggerFactory.getLogger(DataAccessModule.class);

    /**
     * 提供数据库连接池单例
     * 
     * HikariCP 特点：
     * - 高性能连接池，支持高并发
     * - 自动连接验证和回收
     * - 支持连接超时、空闲超时等配置
     * 
     * 游戏服推荐配置：
     * - maximumPoolSize: 20-50（根据数据库并发能力调整）
     * - minimumIdle: 5-10（保持最少连接数）
     * - connectionTimeout: 30000ms（连接超时）
     * - idleTimeout: 600000ms（10 分钟空闲超时）
     * 
     * 线程安全：HikariCP 内部使用 ConcurrentBag 管理连接，完全线程安全
     */
    @Singleton
    @Provides
    static DataSource provideDataSource(GameServerConfig config) {
        logger.info("[DI] 初始化数据库连接池...");
        
        HikariConfig hikariConfig = new HikariConfig();
        hikariConfig.setJdbcUrl(config.getDbUrl());
        hikariConfig.setUsername(config.getDbUser());
        hikariConfig.setPassword(config.getDbPassword());
        
        // 连接池大小配置
        hikariConfig.setMaximumPoolSize(config.getDbPoolSize());
        hikariConfig.setMinimumIdle(Math.max(5, config.getDbPoolSize() / 4));
        
        // 连接超时配置
        hikariConfig.setConnectionTimeout(30000);  // 30 秒
        hikariConfig.setIdleTimeout(600000);       // 10 分钟
        hikariConfig.setMaxLifetime(1800000);      // 30 分钟
        
        // 连接验证
        hikariConfig.setConnectionTestQuery("SELECT 1");
        hikariConfig.setLeakDetectionThreshold(60000);  // 60 秒检测泄漏
        
        // 池名称（便于监控）
        hikariConfig.setPoolName("GameServerPool");
        
        HikariDataSource dataSource = new HikariDataSource(hikariConfig);
        
        logger.info("[DI] 数据库连接池初始化完成: url={}, poolSize={}", 
            config.getDbUrl(), config.getDbPoolSize());
        
        return dataSource;
    }


    @Provides
    @Singleton
    SessionFactory provideSessionFactory(DataSource dataSource, GameServerConfig config) {
        try {
            logger.info("[DI] 初始化 Hibernate SessionFactory...");
            
            // 根据数据库 URL 自动选择方言
            String url = config.getDbUrl();
            String dialect = (url != null && url.contains("postgresql"))
                    ? "org.hibernate.dialect.PostgreSQLDialect"
                    : "org.hibernate.dialect.MySQLDialect";

            Map<String, Object> settings = new HashMap<>();
            settings.put(AvailableSettings.DATASOURCE, dataSource);
            settings.put(AvailableSettings.DIALECT, dialect);
            settings.put(AvailableSettings.HBM2DDL_AUTO, "update");
            settings.put(AvailableSettings.SHOW_SQL, "false");
            settings.put(AvailableSettings.FORMAT_SQL, "true");
            settings.put(AvailableSettings.USE_SECOND_LEVEL_CACHE, "false");
            settings.put("hibernate.jdbc.batch_size", "20");
            settings.put("hibernate.jdbc.fetch_size", "50");

            StandardServiceRegistry registry = new StandardServiceRegistryBuilder()
                    .applySettings(settings)
                    .build();

            SessionFactory sessionFactory = new MetadataSources(registry)
                    .addAnnotatedClass(com.gameserver.entity.Player.class)
                    .addAnnotatedClass(com.gameserver.entity.Backpack.class)
                    .buildMetadata()
                    .buildSessionFactory();
            
            logger.info("[DI] SessionFactory 初始化完成");
            return sessionFactory;
        } catch (Exception e) {
            throw new RuntimeException("Failed to build SessionFactory", e);
        }
    }

    /**
     * 提供 PlayerDAO 单例
     * 
     * 设计说明：
     * - PlayerDAO 是无状态的，可以安全地共享给多个 Actor
     * - 每个 Actor 调用 DAO 方法时，DAO 从连接池获取连接
     * - DAO 方法应该在 blocking-dispatcher 中执行，避免阻塞 Actor 线程
     * 
     * 线程安全：
     * - PlayerDAOImpl 不维护任何状态，只是数据库操作的包装
     * - 所有数据库操作都通过连接池，线程安全
     * - 建议使用 @Blocking 注解标记 DAO 方法，Pekko 会自动切换到 blocking-dispatcher
     */
    @Singleton
    @Provides
    static PlayerDAO providePlayerDAO(SessionFactory sessionFactory) {
        logger.info("[DI] 初始化 PlayerDAO...");
        
        PlayerDAO playerDAO = new PlayerDAOImpl(sessionFactory);
        
        logger.info("[DI] PlayerDAO 初始化完成");
        
        return playerDAO;
    }
}