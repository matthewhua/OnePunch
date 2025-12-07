package com.gameserver.dao;


import com.gameserver.entity.Player;

import java.util.Optional;

/**
 * 玩家数据访问对象接口
 * 
 * 设计原则：
 * 1. 无状态接口，所有方法都是幂等的
 * 2. 由 Dagger2 注入到玩家 Actor 中
 * 3. 所有数据库操作都应该在 blocking-dispatcher 中执行
 * 
 * 游戏服用途：
 * - 玩家登录时查询玩家数据
 * - 玩家属性变化时保存数据
 * - 玩家离线时持久化玩家状态
 * 
 * 线程安全保证：
 * - PlayerDAO 实现类是无状态的
 * - 每个数据库操作都通过连接池获取独立连接
 * - 多个玩家 Actor 可以并发调用同一个 DAO 实例
 */
public interface PlayerDAO {
    
    /**
     * 根据玩家 ID 查询玩家数据
     * 
     * 游戏服用途：玩家登录时调用
     * 
     * @param playerId 玩家 ID
     * @return 玩家数据，如果不存在则返回 Optional.empty()
     */
    Optional<Player> findById(long playerId);
    
    /**
     * 根据玩家账号查询玩家数据
     * 
     * 游戏服用途：玩家登录时调用
     * 
     * @param account 玩家账号
     * @return 玩家数据，如果不存在则返回 Optional.empty()
     */
    Optional<Player> findByAccount(String account);
    
    /**
     * 保存或更新玩家数据
     * 
     * 游戏服用途：
     * - 玩家属性变化时调用（如升级、获得装备等）
     * - 玩家离线时调用，保存最终状态
     * 
     * @param player 玩家数据对象
     * @return true 表示保存成功，false 表示失败
     */
    boolean save(Player player);
    
    /**
     * 删除玩家数据
     * 
     * 游戏服用途：玩家注销账号时调用
     * 
     * @param playerId 玩家 ID
     * @return true 表示删除成功，false 表示失败
     */
    boolean delete(long playerId);
    
    /**
     * 增加玩家经验值
     * 
     * 游戏服用途：玩家击杀怪物或完成任务时调用
     * 
     * @param playerId 玩家 ID
     * @param experience 增加的经验值
     * @return 更新后的玩家等级
     */
    int addExperience(long playerId, int experience);
}
