package com.gameserver.di;

import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import dagger.Module;
import dagger.Provides;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;

import javax.inject.Singleton;
import javax.sql.DataSource;

/**
 * 数据库模块
 * 
 * 职责：
 * 1. 提供 HikariCP 连接池（游戏服高并发必备）
 * 2. 提供 Hibernate SessionFactory
 * 3. 确保单例性和线程安全
 * 
 * 关键配置：
 * - 禁用二级缓存（游戏服多 Actor 并发修改）
 * - 禁用一级缓存自动刷新（避免数据不一致）
 * - 批量操作优化
 */
@Module
public class DatabaseModule {
    
    /**
     * 提供 HikariCP 连接池
     * 
     * 配置说明：
     * - maximumPoolSize: 最大连接数（游戏服高并发）
     * - minimumIdle: 最小空闲连接数
     * - connectionTimeout: 获取连接超时时间
     * - idleTimeout: 连接空闲超时时间
     * - maxLifetime: 连接最大生命周期
     * - leakDetectionThreshold: 连接泄漏检测阈值
     */
    @Singleton
    @Provides
    static DataSource provideDataSource() {
        HikariConfig config = new HikariConfig();
        
        // PostgreSQL 连接配置
        config.setJdbcUrl("jdbc:postgresql://localhost:5432/gamedb");
        config.setUsername("postgres");
        config.setPassword("password");
        config.setDriverClassName("org.postgresql.Driver");
        
        // 连接池配置（游戏服高并发必备）
        config.setMaximumPoolSize(20);      // 最大连接数
        config.setMinimumIdle(5);           // 最小空闲连接
        config.setConnectionTimeout(30000); // 获取连接超时（30 秒）
        config.setIdleTimeout(600000);      // 连接空闲超时（10 分钟）
        config.setMaxLifetime(1800000);     // 连接最大生命周期（30 分钟）
        
        // 连接验证
        config.setConnectionTestQuery("SELECT 1");
        config.setLeakDetectionThreshold(60000);  // 60 秒检测泄漏
        
        // 线程池配置
        config.setThreadFactory(r -> {
            Thread t = new Thread(r, "HikariCP-GameServer");
            t.setDaemon(false);
            return t;
        });
        
        return new HikariDataSource(config);
    }
    
    /**
     * 提供 Hibernate SessionFactory
     * 
     * 关键配置：
     * 1. 禁用二级缓存（游戏服多 Actor 并发修改会导致数据不一致）
     * 2. 禁用一级缓存自动刷新（避免过期数据）
     * 3. 批量操作优化（jdbc.batch_size）
     * 4. 使用 PostgreSQL 方言
     */
    @Singleton
    @Provides
    static SessionFactory provideSessionFactory(DataSource dataSource) {
        Configuration config = new Configuration();
        
        // 加载 hibernate.cfg.xml
        config.configure("hibernate.cfg.xml");
        
        // 关键：禁用二级缓存（游戏服多 Actor 并发修改）
        config.setProperty("hibernate.cache.use_second_level_cache", "false");
        
        // 关键：禁用一级缓存自动刷新（避免过期数据）
        config.setProperty("hibernate.enable_lazy_load_no_trans", "false");
        
        // 批量操作优化
        config.setProperty("hibernate.jdbc.batch_size", "20");
        config.setProperty("hibernate.jdbc.fetch_size", "50");
        
        // PostgreSQL 方言
        config.setProperty("hibernate.dialect", "org.hibernate.dialect.PostgreSQLDialect");
        
        // 日志配置
        config.setProperty("hibernate.show_sql", "false");
        config.setProperty("hibernate.format_sql", "true");
        
        // 构建 SessionFactory
        return config.buildSessionFactory();
    }
}
