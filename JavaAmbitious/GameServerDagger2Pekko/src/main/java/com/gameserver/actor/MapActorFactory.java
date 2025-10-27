package com.gameserver.actor;

import com.gameserver.service.MapService;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Props;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;
import javax.inject.Provider;

/**
 * 地图 Actor 工厂
 * 
 * 设计目标（解决游戏服痛点）：
 * 1. 支持动态创建地图 Actor
 *    - 玩家进入地图时创建 Actor
 *    - 玩家全部离开时销毁 Actor
 *    - 避免启动时创建所有地图，浪费内存
 * 
 * 2. 使用 Provider 实现延迟初始化
 *    - 地图 Actor 只在需要时创建
 *    - 减少服务器启动时间和内存占用
 * 
 * 3. 确保依赖的单例性
 *    - 所有地图 Actor 共享同一个 MapService
 *    - 避免重复创建，节省内存
 * 
 * 使用示例：
 *   MapActorFactory factory = component.getMapActorFactory();
 *   ActorRef mapActor = factory.create(mapId);
 *   mapActor.tell(new MapMessage.PlayerEnter(...), ActorRef.noSender());
 */
public class MapActorFactory {
    
    private static final Logger logger = LoggerFactory.getLogger(MapActorFactory.class);
    
    private final ActorSystem actorSystem;
    private final Provider<MapService> mapServiceProvider;

    /**
     * 构造函数
     * 
     * 使用 Provider<MapService> 实现延迟初始化
     * - 地图 Actor 创建时才会调用 mapServiceProvider.get()
     * - 避免启动时初始化所有依赖
     * 
     * @param actorSystem Pekko ActorSystem（由 Dagger2 注入）
     * @param mapServiceProvider 地图服务 Provider（由 Dagger2 注入）
     */
    @Inject
    public MapActorFactory(ActorSystem actorSystem, Provider<MapService> mapServiceProvider) {
        this.actorSystem = actorSystem;
        this.mapServiceProvider = mapServiceProvider;
        
        logger.info("[Factory] MapActorFactory 初始化完成");
    }

    /**
     * 创建地图 Actor
     * 
     * 游戏服用途：
     * - 玩家进入地图时调用
     * - 返回 ActorRef，用于发送消息给地图 Actor
     * 
     * 线程安全保证：
     * - ActorSystem.actorOf() 是线程安全的
     * - 返回的 ActorRef 也是线程安全的
     * - 多个线程可以并发调用，无竞态条件
     * 
     * @param mapId 地图 ID
     * @return 地图 Actor 的引用
     */
    public ActorRef create(int mapId) {
        logger.info("[Factory] 创建地图 Actor: mapId={}", mapId);
        
        // 获取 MapService 实例
        // 如果是 @Singleton，这里会返回单例
        // 如果是 @Prototype，这里会创建新实例
        MapService mapService = mapServiceProvider.get();
        
        // 创建 Props 对象
        Props props = Props.create(
            MapActor.class,
            mapId,
            mapService
        );
        
        // 创建 Actor 实例
        ActorRef mapActor = actorSystem.actorOf(
            props,
            "map-" + mapId
        );
        
        logger.info("[Factory] 地图 Actor 创建成功: mapId={}, actorRef={}", mapId, mapActor);
        
        return mapActor;
    }
}
