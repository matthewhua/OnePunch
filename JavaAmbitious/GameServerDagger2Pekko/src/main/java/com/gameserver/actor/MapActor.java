package com.gameserver.actor;

import com.gameserver.service.MapService;
import org.apache.pekko.actor.AbstractActor;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashSet;
import java.util.Set;

/**
 * 地图 Actor
 * 
 * 设计目标：
 * 1. 每个地图对应一个 Actor 实例
 * 2. 管理地图中的所有玩家和怪物
 * 3. 处理地图中的事件（玩家进入、离开、怪物死亡等）
 * 
 * 线程安全保证：
 * 1. Actor 本身是单线程的：
 *    - 同一个 Actor 的消息处理是串行的
 *    - 不同 Actor 的消息处理可以并行
 *    - Pekko 框架保证消息顺序
 * 
 * 2. 注入的依赖是线程安全的：
 *    - MapService：无状态，线程安全
 * 
 * 3. 地图的动态状态由 Actor 管理：
 *    - playerSet：当前地图中的玩家集合
 *    - 只在 Actor 的单线程中修改
 *    - 不需要加锁或同步
 */
public class MapActor extends AbstractActor {
    
    private static final Logger logger = LoggerFactory.getLogger(MapActor.class);
    
    private final int mapId;
    private final MapService mapService;
    
    // 地图的动态状态（由 Actor 维护）
    private final Set<Long> playerSet = new HashSet<>();

    /**
     * 构造函数
     * 
     * @param mapId 地图 ID
     * @param mapService 地图服务（由 Dagger2 注入）
     */
    public MapActor(int mapId, MapService mapService) {
        this.mapId = mapId;
        this.mapService = mapService;
        
        logger.info("[MapActor] 地图 Actor 创建: mapId={}", mapId);
    }

    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(MapMessage.PlayerEnter.class, this::handlePlayerEnter)
            .match(MapMessage.PlayerLeave.class, this::handlePlayerLeave)
            .match(MapMessage.GetPlayerCount.class, this::handleGetPlayerCount)
            .matchAny(msg -> logger.warn("[MapActor] 收到未知消息: {}", msg))
            .build();
    }

    /**
     * 处理玩家进入地图消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerSet 只在这个 Actor 中修改
     * - 不需要加锁或同步
     */
    private void handlePlayerEnter(MapMessage.PlayerEnter msg) {
        logger.info("[MapActor] 处理玩家进入: mapId={}, playerId={}", mapId, msg.playerId);
        
        playerSet.add(msg.playerId);
        
        logger.info("[MapActor] 玩家进入成功: mapId={}, playerCount={}", mapId, playerSet.size());
        
        // 发送响应
        sender().tell(new MapMessage.PlayerEnterResponse(true), self());
    }

    /**
     * 处理玩家离开地图消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerSet 只在这个 Actor 中修改
     * - 不需要加锁或同步
     */
    private void handlePlayerLeave(MapMessage.PlayerLeave msg) {
        logger.info("[MapActor] 处理玩家离开: mapId={}, playerId={}", mapId, msg.playerId);
        
        playerSet.remove(msg.playerId);
        
        logger.info("[MapActor] 玩家离开成功: mapId={}, playerCount={}", mapId, playerSet.size());
        
        // 如果地图中没有玩家，可以考虑销毁 Actor 以释放内存
        if (playerSet.isEmpty()) {
            logger.info("[MapActor] 地图中没有玩家，可以销毁 Actor: mapId={}", mapId);
            // context().stop(self());  // 可选：自动销毁
        }
        
        // 发送响应
        sender().tell(new MapMessage.PlayerLeaveResponse(true), self());
    }

    /**
     * 处理获取玩家数量消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerSet 是只读操作
     * - 不需要加锁或同步
     */
    private void handleGetPlayerCount(MapMessage.GetPlayerCount msg) {
        logger.debug("[MapActor] 获取玩家数量: mapId={}, playerCount={}", mapId, playerSet.size());
        
        sender().tell(new MapMessage.GetPlayerCountResponse(playerSet.size()), self());
    }
}
