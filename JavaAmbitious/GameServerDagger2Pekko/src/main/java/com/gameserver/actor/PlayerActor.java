package com.gameserver.actor;

import com.gameserver.dao.PlayerDAO;
import com.gameserver.model.Player;
import com.gameserver.service.SkillService;
import org.apache.pekko.actor.AbstractActor;
import org.apache.pekko.actor.ActorRef;
import org.apache.pekko.dispatch.Dispatchers;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Optional;

/**
 * 玩家 Actor
 * 
 * 设计目标：
 * 1. 每个玩家对应一个 Actor 实例
 * 2. 玩家的所有操作（攻击、移动、升级等）都通过消息驱动
 * 3. Actor 本身是单线程的，不需要加锁
 * 
 * 线程安全保证：
 * 1. Actor 本身是单线程的：
 *    - 同一个 Actor 的消息处理是串行的
 *    - 不同 Actor 的消息处理可以并行
 *    - Pekko 框架保证消息顺序
 * 
 * 2. 注入的依赖是线程安全的：
 *    - PlayerDAO：使用连接池，线程安全
 *    - SkillService：无状态，线程安全
 * 
 * 3. 避免竞态条件的关键：
 *    - 不在 Actor 中维护可变状态（除了玩家数据）
 *    - 所有外部调用都通过消息，不直接访问字段
 *    - 数据库操作使用 blocking-dispatcher，避免阻塞 Actor 线程
 * 
 * 游戏服最佳实践：
 * - 玩家的属性变化应该通过消息驱动
 * - 数据库操作应该使用 blocking-dispatcher
 * - 避免在 Actor 中执行长时间操作
 */
public class PlayerActor extends AbstractActor {
    
    private static final Logger logger = LoggerFactory.getLogger(PlayerActor.class);
    
    private final long playerId;
    private final PlayerDAO playerDAO;
    private final SkillService skillService;
    
    // 玩家的当前状态（由 Actor 维护）
    private Player playerData;

    /**
     * 构造函数
     * 
     * 此处注入玩家 DAO，避免硬编码依赖
     * 由 PlayerActorFactory 调用，确保注入的是单例实例
     * 
     * @param playerId 玩家 ID
     * @param playerDAO 玩家数据访问对象（由 Dagger2 注入）
     * @param skillService 技能系统服务（由 Dagger2 注入）
     */
    public PlayerActor(long playerId, PlayerDAO playerDAO, SkillService skillService) {
        this.playerId = playerId;
        this.playerDAO = playerDAO;
        this.skillService = skillService;
        
        logger.info("[PlayerActor] 玩家 Actor 创建: playerId={}", playerId);
    }

    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerMessage.Login.class, this::handleLogin)
            .match(PlayerMessage.Attack.class, this::handleAttack)
            .match(PlayerMessage.AddExperience.class, this::handleAddExperience)
            .match(PlayerMessage.Logout.class, this::handleLogout)
            .matchAny(msg -> logger.warn("[PlayerActor] 收到未知消息: {}", msg))
            .build();
    }

    /**
     * 处理玩家登录消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerDAO.findById() 是线程安全的（使用连接池）
     * - 但 findById() 是阻塞操作，应该在 blocking-dispatcher 中执行
     * 
     * 游戏服最佳实践：
     * - 使用 pipe() 或 ask() 模式处理异步数据库操作
     * - 或者使用 blocking-dispatcher 来处理阻塞操作
     */
    private void handleLogin(PlayerMessage.Login msg) {
        logger.info("[PlayerActor] 处理登录: playerId={}, account={}", playerId, msg.account);
        
        // 从数据库查询玩家数据
        // 注意：这是一个阻塞操作，实际应该使用 blocking-dispatcher
        Optional<Player> playerOpt = playerDAO.findById(playerId);
        
        if (playerOpt.isPresent()) {
            playerData = playerOpt.get();
            logger.info("[PlayerActor] 玩家登录成功: {}", playerData);
            
            // 发送登录成功消息
            sender().tell(new PlayerMessage.LoginResponse(true, playerData), self());
        } else {
            logger.warn("[PlayerActor] 玩家不存在: playerId={}", playerId);
            
            // 发送登录失败消息
            sender().tell(new PlayerMessage.LoginResponse(false, null), self());
        }
    }

    /**
     * 处理玩家攻击消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - skillService.calculateDamage() 是线程安全的（无状态）
     * - 多个玩家 Actor 可以并发调用同一个 skillService 实例
     */
    private void handleAttack(PlayerMessage.Attack msg) {
        if (playerData == null) {
            logger.warn("[PlayerActor] 玩家未登录，无法攻击: playerId={}", playerId);
            return;
        }
        
        logger.info("[PlayerActor] 处理攻击: playerId={}, targetId={}, skillId={}", 
            playerId, msg.targetId, msg.skillId);
        
        // 计算伤害
        // 注意：skillService 是单例，多个玩家 Actor 可以并发调用
        int damage = skillService.calculateDamage(playerId, msg.skillId, msg.targetId);
        
        logger.info("[PlayerActor] 攻击伤害: playerId={}, targetId={}, damage={}", 
            playerId, msg.targetId, damage);
        
        // 发送攻击结果
        sender().tell(new PlayerMessage.AttackResponse(damage), self());
    }

    /**
     * 处理玩家增加经验消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerDAO.addExperience() 是线程安全的（使用连接池）
     * - 但 addExperience() 是阻塞操作，应该在 blocking-dispatcher 中执行
     */
    private void handleAddExperience(PlayerMessage.AddExperience msg) {
        if (playerData == null) {
            logger.warn("[PlayerActor] 玩家未登录，无法增加经验: playerId={}", playerId);
            return;
        }
        
        logger.info("[PlayerActor] 处理增加经验: playerId={}, experience={}", 
            playerId, msg.experience);
        
        // 增加经验值
        // 注意：这是一个阻塞操作，实际应该使用 blocking-dispatcher
        int newLevel = playerDAO.addExperience(playerId, msg.experience);
        
        logger.info("[PlayerActor] 经验增加成功: playerId={}, newLevel={}", playerId, newLevel);
        
        // 发送响应
        sender().tell(new PlayerMessage.AddExperienceResponse(newLevel), self());
    }

    /**
     * 处理玩家登出消息
     * 
     * 线程安全说明：
     * - 此方法在 Actor 的单线程中执行
     * - playerDAO.save() 是线程安全的（使用连接池）
     * - 但 save() 是阻塞操作，应该在 blocking-dispatcher 中执行
     */
    private void handleLogout(PlayerMessage.Logout msg) {
        logger.info("[PlayerActor] 处理登出: playerId={}", playerId);
        
        if (playerData != null) {
            // 保存玩家数据
            // 注意：这是一个阻塞操作，实际应该使用 blocking-dispatcher
            boolean success = playerDAO.save(playerData);
            
            if (success) {
                logger.info("[PlayerActor] 玩家数据保存成功: playerId={}", playerId);
            } else {
                logger.error("[PlayerActor] 玩家数据保存失败: playerId={}", playerId);
            }
        }
        
        // 停止 Actor
        context().stop(self());
    }
}
