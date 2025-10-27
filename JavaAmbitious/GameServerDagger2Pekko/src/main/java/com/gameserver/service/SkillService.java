package com.gameserver.service;

/**
 * 技能系统服务接口
 * 
 * 设计原则：
 * 1. 无状态服务，所有方法都是幂等的
 * 2. 由 Dagger2 注入到玩家 Actor 中
 * 3. 多个玩家 Actor 可以并发调用同一个实例
 * 
 * 游戏服用途：
 * - 计算技能伤害（基础伤害 + 属性加成 + 暴击等）
 * - 管理技能冷却时间
 * - 计算能量消耗和恢复
 * 
 * 线程安全保证：
 * - SkillService 实现类是无状态的
 * - 所有计算都基于输入参数，不依赖内部状态
 * - 多个玩家 Actor 可以并发调用，无竞态条件
 */
public interface SkillService {
    
    /**
     * 计算技能伤害
     * 
     * 游戏服用途：玩家使用技能时调用
     * 
     * @param playerId 玩家 ID
     * @param skillId 技能 ID
     * @param targetId 目标 ID
     * @return 计算后的伤害值
     */
    int calculateDamage(long playerId, int skillId, long targetId);
    
    /**
     * 获取技能冷却时间
     * 
     * 游戏服用途：玩家查询技能是否可用时调用
     * 
     * @param skillId 技能 ID
     * @return 冷却时间（毫秒）
     */
    long getSkillCooldown(int skillId);
    
    /**
     * 计算技能能量消耗
     * 
     * 游戏服用途：玩家使用技能前检查能量是否足够
     * 
     * @param skillId 技能 ID
     * @return 能量消耗值
     */
    int getSkillEnergyCost(int skillId);
}
