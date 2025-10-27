package com.gameserver.service.impl;

import com.gameserver.service.SkillService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * 技能系统服务实现
 * <p>
 * 设计说明：
 * 1. 无状态实现，所有计算都基于输入参数
 * 2. 由 Dagger2 管理单例，所有玩家 Actor 共享
 * 3. 多个玩家 Actor 可以并发调用，无竞态条件
 * <p>
 * 线程安全保证：
 * - 不维护任何可变状态
 * - 所有计算都是原子操作
 * - 多个线程可以并发调用，无竞态条件
 */
public class SkillServiceImpl implements SkillService {

    private static final Logger logger = LoggerFactory.getLogger(SkillServiceImpl.class);

    @Override
    public int calculateDamage(long playerId, int skillId, long targetId) {
        // 此处注入技能伤害计算逻辑
        // 实际游戏服应该从数据库查询技能配置，然后计算伤害

        // 示例：基础伤害 + 随机浮动
        int baseDamage = 50;
        int randomDamage = (int) (Math.random() * 20);
        int totalDamage = baseDamage + randomDamage;

        logger.debug("[SkillService] 计算伤害: playerId={}, skillId={}, targetId={}, damage={}",
                playerId, skillId, targetId, totalDamage);

        return totalDamage;
    }

    @Override
    public long getSkillCooldown(int skillId) {
        // 示例：不同技能的冷却时间
        switch (skillId) {
            case 1:
                return 5000; // 5秒
            case 2:
                return 8000; // 8秒
            case 3:
                return 10000; // 10秒
            default:
                return 0; // 默认无冷却
        }
    }

    @Override
    public int getSkillEnergyCost(int skillId) {
        // 示例：不同技能的能量消耗
        switch (skillId) {
            case 1:
                return 10;
            case 2:
                return 20;
            case 3:
                return 50;
            default:
                return 5;
        }
    }
}
