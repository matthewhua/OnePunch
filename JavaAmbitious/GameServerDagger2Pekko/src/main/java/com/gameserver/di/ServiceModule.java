package com.gameserver.di;

import com.gameserver.service.SkillService;
import com.gameserver.service.MapService;
import com.gameserver.service.impl.SkillServiceImpl;
import com.gameserver.service.impl.MapServiceImpl;
import dagger.Module;
import dagger.Provides;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.inject.Singleton;

/**
 * 业务服务层依赖注入模块
 * 
 * 设计目标：
 * 1. 提供游戏逻辑服务的单例（技能系统、地图系统等）
 * 2. 这些服务被多个 Actor 共享调用
 * 3. 确保服务的线程安全性
 * 
 * 游戏服痛点解决：
 * - 问题：每个玩家 Actor 创建自己的 SkillService 实例，导致重复计算和内存浪费
 * - 方案：使用 @Singleton 共享服务实例，所有 Actor 调用同一个实例
 * 
 * 线程安全保证：
 * - 所有 Service 实现都是无状态的（不维护玩家特定的数据）
 * - Service 方法接收参数，返回结果，不修改内部状态
 * - 多个 Actor 可以并发调用同一个 Service 实例，完全安全
 */
@Module
public class ServiceModule {
    
    private static final Logger logger = LoggerFactory.getLogger(ServiceModule.class);

    /**
     * 提供技能系统服务单例
     * 
     * SkillService 职责：
     * - 计算技能伤害（基础伤害 + 属性加成 + 暴击等）
     * - 管理技能冷却时间
     * - 计算能量消耗和恢复
     * 
     * 游戏服设计：
     * - SkillService 是无状态的，只提供计算方法
     * - 玩家的技能冷却状态保存在玩家 Actor 中
     * - 多个玩家 Actor 可以并发调用同一个 SkillService，无竞态条件
     * 
     * 线程安全：
     * - SkillService 不维护任何可变状态
     * - 所有计算都基于输入参数，不依赖内部状态
     * - 可以安全地被多个线程并发调用
     * 
     * 示例用法（在玩家 Actor 中）：
     *   SkillService skillService = ...;  // 注入的单例
     *   int damage = skillService.calculateDamage(playerId, skillId, targetId);
     *   // 多个玩家 Actor 并发调用，完全安全
     */
    @Singleton
    @Provides
    static SkillService provideSkillService() {
        logger.info("[DI] 初始化 SkillService...");
        
        SkillService skillService = new SkillServiceImpl();
        
        logger.info("[DI] SkillService 初始化完成");
        
        return skillService;
    }

    /**
     * 提供地图系统服务单例
     * 
     * MapService 职责：
     * - 管理地图的基础信息（怪物分布、资源点等）
     * - 计算玩家在地图中的移动和碰撞
     * - 管理地图的事件系统
     * 
     * 游戏服设计：
     * - MapService 提供地图的静态信息和计算方法
     * - 地图的动态状态（当前玩家、怪物实例等）由地图 Actor 管理
     * - 多个地图 Actor 可以共享同一个 MapService
     * 
     * 线程安全：
     * - MapService 不维护任何可变状态
     * - 地图的动态状态由地图 Actor 管理，Actor 本身是单线程的
     * - 可以安全地被多个地图 Actor 并发调用
     * 
     * 示例用法（在地图 Actor 中）：
     *   MapService mapService = ...;  // 注入的单例
     *   boolean canMove = mapService.canMoveTo(mapId, x, y);
     *   // 多个地图 Actor 并发调用，完全安全
     */
    @Singleton
    @Provides
    static MapService provideMapService() {
        logger.info("[DI] 初始化 MapService...");
        
        MapService mapService = new MapServiceImpl();
        
        logger.info("[DI] MapService 初始化完成");
        
        return mapService;
    }
}
