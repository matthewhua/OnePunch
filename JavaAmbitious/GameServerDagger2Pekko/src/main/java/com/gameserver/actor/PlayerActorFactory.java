package com.gameserver.actor;

import com.gameserver.dao.PlayerDAO;
import com.gameserver.service.SkillService;
import dagger.assisted.Assisted;
import dagger.assisted.AssistedFactory;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.actor.ActorSystem;
import org.apache.pekko.actor.Props;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Inject;

/**
 * 玩家 Actor 工厂
 * 
 * 设计目标（解决游戏服痛点）：
 * 1. 避免 "构造器地狱"：PlayerActor 需要 PlayerDAO、SkillService 等多个依赖
 *    - 传统做法：new PlayerActor(dao, skillService, ...) 导致耦合
 *    - Dagger2 方案：工厂已注入所有依赖，调用方只需 factory.create(playerId)
 * 
 * 2. 确保依赖的单例性：所有玩家 Actor 共享同一个 PlayerDAO 和 SkillService
 *    - 避免重复创建，节省内存
 *    - 避免数据不一致
 * 
 * 3. 线程安全：工厂方法本身是线程安全的
 *    - 多个线程可以并发调用 create()
 *    - 返回的 ActorRef 也是线程安全的
 * 
 * 使用示例：
 *   PlayerActorFactory factory = component.getPlayerActorFactory();
 *   ActorRef playerActor = factory.create(playerId);
 *   playerActor.tell(new PlayerMessage.Login(...), ActorRef.noSender());
 */
public class PlayerActorFactory {
    
    private static final Logger logger = LoggerFactory.getLogger(PlayerActorFactory.class);
    
    private final ActorSystem actorSystem;
    private final PlayerDAO playerDAO;
    private final SkillService skillService;

    /**
     * 构造函数
     * 
     * 此处注入玩家 DAO，避免硬编码依赖
     * 由 Dagger2 自动调用，确保注入的是单例实例
     * 
     * @param actorSystem Pekko ActorSystem（由 Dagger2 注入）
     * @param playerDAO 玩家数据访问对象（由 Dagger2 注入）
     * @param skillService 技能系统服务（由 Dagger2 注入）
     */
    @Inject
    public PlayerActorFactory(ActorSystem actorSystem, PlayerDAO playerDAO, SkillService skillService) {
        this.actorSystem = actorSystem;
        this.playerDAO = playerDAO;
        this.skillService = skillService;
        
        logger.info("[Factory] PlayerActorFactory 初始化完成");
    }

    /**
     * 创建玩家 Actor
     * 
     * 游戏服用途：
     * - 玩家登录时调用，创建玩家 Actor
     * - 返回 ActorRef，用于发送消息给玩家 Actor
     * 
     * 线程安全保证：
     * - ActorSystem.actorOf() 是线程安全的
     * - 返回的 ActorRef 也是线程安全的
     * - 多个线程可以并发调用，无竞态条件
     * 
     * @param playerId 玩家 ID
     * @return 玩家 Actor 的引用
     */
    public ActorRef create(long playerId) {
        logger.info("[Factory] 创建玩家 Actor: playerId={}", playerId);
        
        // 创建 Props 对象，指定 Actor 类和构造参数
        // 关键：playerDAO 和 skillService 已由 Dagger2 注入，这里直接使用
        Props props = Props.create(
            PlayerActor.class,
            playerId,
            playerDAO,
            skillService
        );
        
        // 创建 Actor 实例
        // ActorSystem.actorOf() 会在 ActorSystem 的线程池中创建 Actor
        // 返回的 ActorRef 是线程安全的
        ActorRef playerActor = actorSystem.actorOf(
            props,
            "player-" + playerId  // Actor 名称
        );
        
        logger.info("[Factory] 玩家 Actor 创建成功: playerId={}, actorRef={}", playerId, playerActor);
        
        return playerActor;
    }
}
