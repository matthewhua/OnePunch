package com.gameserver.di;

import com.gameserver.actor.PlayerActorFactory;
import com.gameserver.actor.MapActorFactory;
import com.gameserver.config.GameServerConfig;
import com.gameserver.dao.PlayerDAO;
import com.gameserver.service.SkillService;
import com.gameserver.service.MapService;
import dagger.Component;
import org.apache.pekko.actor.ActorSystem;

import javax.inject.Singleton;

/**
 * 游戏服务器核心 Dagger2 组件定义
 * 
 * 设计目标：
 * 1. 统一管理游戏服的所有依赖（ActorSystem、DAO、Service 等）
 * 2. 确保 ActorSystem 和数据库连接池等资源为单例，避免重复创建
 * 3. 为 Actor 工厂提供依赖注入入口，避免硬编码依赖
 * 4. 支持游戏服的分布式扩展（Pekko Cluster）
 * 
 * 游戏服痛点解决：
 * - 问题：多个 Actor 需要相同的依赖（如 PlayerDAO），手动创建导致耦合
 * - 方案：通过 Dagger2 @Singleton 确保单一实例，所有 Actor 共享
 * 
 * 线程安全保证：
 * - Dagger2 生成的代码在编译时已确定依赖关系，运行时无竞态条件
 * - ActorSystem 本身线程安全，由 Pekko 框架保证
 * - DAO 和 Service 的线程安全由具体实现保证（使用连接池）
 */
@Singleton
@Component(modules = {
    GameServerModule.class,
    ActorSystemModule.class,
    DataAccessModule.class,
    ServiceModule.class,
})
public interface GameServerComponent {

    // ============ 核心基础设施 ============
    
    /**
     * 获取 ActorSystem 单例
     * 
     * 游戏服用途：
     * - 创建所有 Actor（玩家、地图、副本等）
     * - 管理 Actor 的生命周期
     * - 支持 Pekko Cluster 分布式部署
     * 
     * 线程安全：ActorSystem 内部使用线程池，所有操作都是线程安全的
     */
    ActorSystem getActorSystem();

    /**
     * 获取游戏服配置单例
     * 
     * 包含：
     * - 服务器端口、IP 配置
     * - Actor 线程池大小
     * - 数据库连接参数
     * - 游戏逻辑参数（如伤害系数、冷却时间等）
     */
    GameServerConfig getGameServerConfig();

    // ============ 数据访问层 ============
    
    /**
     * 获取玩家 DAO 单例
     * 
     * 游戏服用途：
     * - 玩家 Actor 查询和保存玩家数据
     * - 支持玩家登录、属性查询、数据持久化
     * 
     * 线程安全：DAO 使用数据库连接池（HikariCP），支持并发访问
     */
    PlayerDAO getPlayerDAO();

    // ============ 业务服务层 ============
    
    /**
     * 获取技能系统服务单例
     * 
     * 游戏服用途：
     * - 玩家 Actor 调用技能逻辑
     * - 计算伤害、冷却、能量消耗等
     * 
     * 线程安全：Service 内部使用不可变数据结构和原子操作
     */
    SkillService getSkillService();

    /**
     * 获取地图系统服务单例
     * 
     * 游戏服用途：
     * - 地图 Actor 管理地图状态
     * - 处理玩家进入/离开地图的逻辑
     */
    MapService getMapService();

    // ============ Actor 工厂（关键依赖注入入口） ============
    
    /**
     * 获取玩家 Actor 工厂
     * 
     * 设计目的：
     * - 工厂已注入所有依赖（PlayerDAO、SkillService 等）
     * - 调用方只需调用 factory.create(playerId)，无需手动传递依赖
     * - 避免 "构造器地狱"：PlayerActorFactory 内部处理所有依赖组装
     * 
     * 使用示例：
     *   PlayerActorFactory factory = component.getPlayerActorFactory();
     *   ActorRef playerActor = factory.create(playerId);
     * 
     * 线程安全：工厂方法本身是线程安全的，返回的 ActorRef 也是线程安全的
     */
    PlayerActorFactory getPlayerActorFactory();

    /**
     * 获取地图 Actor 工厂
     * 
     * 设计目的：
     * - 地图 Actor 创建时注入 MapService、ActorSystem 等依赖
     * - 支持动态创建地图（玩家进入时创建，离开时销毁）
     */
    MapActorFactory getMapActorFactory();
}