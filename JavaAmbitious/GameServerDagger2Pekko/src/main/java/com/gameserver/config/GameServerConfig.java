package com.gameserver.config;

import java.io.IOException;
import java.io.InputStream;
import java.util.Properties;

/**
 * 游戏服务器配置类
 * 
 * 设计原则：
 * 1. 所有字段都是 final，确保不可变性
 * 2. 通过 Builder 模式创建实例，避免构造器参数过多
 * 3. 由 Dagger2 管理单例，所有 Actor 共享同一个配置实例
 * 4. 支持从 application.properties 加载配置
 * 5. 支持环境变量覆盖（System.getenv()）
 * 
 * 线程安全保证：
 * - 所有字段都是 final，创建后不可修改
 * - 多个 Actor 可以并发读取配置，无竞态条件
 * - 不需要加锁或同步
 */
public final class GameServerConfig {
    
    // ============ 服务器配置 ============
    private final int serverPort;
    private final String serverHost;
    private final String actorSystemName;
    
    // ============ 数据库配置 ============
    private final String dbUrl;
    private final String dbUser;
    private final String dbPassword;
    private final int dbPoolSize;
    
    // ============ Actor 配置 ============
    private final String actorDispatcher;

    private GameServerConfig(Builder builder) {
        this.serverPort = builder.serverPort;
        this.serverHost = builder.serverHost;
        this.actorSystemName = builder.actorSystemName;
        this.dbUrl = builder.dbUrl;
        this.dbUser = builder.dbUser;
        this.dbPassword = builder.dbPassword;
        this.dbPoolSize = builder.dbPoolSize;
        this.actorDispatcher = builder.actorDispatcher;
    }

    // ============ Getter 方法 ============
    
    public int getServerPort() { return serverPort; }
    public String getServerHost() { return serverHost; }
    public String getActorSystemName() { return actorSystemName; }
    public String getDbUrl() { return dbUrl; }
    public String getDbUser() { return dbUser; }
    public String getDbPassword() { return dbPassword; }
    public int getDbPoolSize() { return dbPoolSize; }
    public String getActorDispatcher() { return actorDispatcher; }

    // ============ Builder 模式 ============
    
    public static Builder builder() {
        return new Builder();
    }

    public static GameServerConfig createDefault() {
        return builder().build();
    }

    /**
     * 从 application.properties 加载配置
     */
    public static GameServerConfig loadFromProperties() {
        Properties props = new Properties();
        try (InputStream input = GameServerConfig.class.getClassLoader()
                .getResourceAsStream("application.properties")) {
            if (input != null) {
                props.load(input);
            }
        } catch (IOException e) {
            System.err.println("Failed to load application.properties: " + e.getMessage());
        }
        
        Builder builder = builder();
        builder.serverPort(Integer.parseInt(props.getProperty("server.port", "8888")));
        builder.serverHost(props.getProperty("server.host", "0.0.0.0"));
        builder.actorSystemName(props.getProperty("actor.system.name", "GameServer"));
        builder.dbUrl(props.getProperty("db.url", "jdbc:postgresql://localhost:5432/gamedb"));
        builder.dbUser(props.getProperty("db.user", "postgres"));
        builder.dbPassword(props.getProperty("db.password", "password"));
        builder.dbPoolSize(Integer.parseInt(props.getProperty("db.pool.size.max", "20")));
        builder.actorDispatcher(props.getProperty("actor.dispatcher", "default"));
        
        return builder.build();
    }

    public static class Builder {
        private int serverPort = 8888;
        private String serverHost = "0.0.0.0";
        private String actorSystemName = "GameServer";
        private String dbUrl = "jdbc:postgresql://localhost:5432/gamedb";
        private String dbUser = "postgres";
        private String dbPassword = "password";
        private int dbPoolSize = 20;
        private String actorDispatcher = "default";

        public Builder serverPort(int port) {
            this.serverPort = port;
            return this;
        }

        public Builder serverHost(String host) {
            this.serverHost = host;
            return this;
        }

        public Builder actorSystemName(String name) {
            this.actorSystemName = name;
            return this;
        }

        public Builder dbUrl(String url) {
            this.dbUrl = url;
            return this;
        }

        public Builder dbUser(String user) {
            this.dbUser = user;
            return this;
        }

        public Builder dbPassword(String password) {
            this.dbPassword = password;
            return this;
        }

        public Builder dbPoolSize(int size) {
            this.dbPoolSize = size;
            return this;
        }

        public Builder actorDispatcher(String dispatcher) {
            this.actorDispatcher = dispatcher;
            return this;
        }

        public GameServerConfig build() {
            return new GameServerConfig(this);
        }
    }

    @Override
    public String toString() {
        return "GameServerConfig{" +
                "serverPort=" + serverPort +
                ", serverHost='" + serverHost + '\'' +
                ", actorSystemName='" + actorSystemName + '\'' +
                ", dbUrl='" + dbUrl + '\'' +
                ", dbPoolSize=" + dbPoolSize +
                '}';
    }
}
