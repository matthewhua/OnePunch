package com.gameserver.service;

/**
 * 地图系统服务接口
 * 
 * 设计原则：
 * 1. 无状态服务，提供地图的静态信息和计算方法
 * 2. 由 Dagger2 注入到地图 Actor 中
 * 3. 多个地图 Actor 可以并发调用同一个实例
 * 
 * 游戏服用途：
 * - 管理地图的基础信息（怪物分布、资源点等）
 * - 计算玩家在地图中的移动和碰撞
 * - 管理地图的事件系统
 * 
 * 线程安全保证：
 * - MapService 实现类是无状态的
 * - 所有计算都基于输入参数，不依赖内部状态
 * - 多个地图 Actor 可以并发调用，无竞态条件
 */
public interface MapService {
    
    /**
     * 检查玩家是否可以移动到指定位置
     * 
     * 游戏服用途：玩家移动时检查是否有障碍物
     * 
     * @param mapId 地图 ID
     * @param x X 坐标
     * @param y Y 坐标
     * @return true 表示可以移动，false 表示有障碍物
     */
    boolean canMoveTo(int mapId, int x, int y);
    
    /**
     * 获取地图的怪物列表
     * 
     * 游戏服用途：地图 Actor 初始化时调用，获取该地图的所有怪物
     * 
     * @param mapId 地图 ID
     * @return 怪物 ID 列表
     */
    int[] getMonsterList(int mapId);
    
    /**
     * 获取地图的资源点列表
     * 
     * 游戏服用途：地图 Actor 初始化时调用，获取该地图的所有资源点
     * 
     * @param mapId 地图 ID
     * @return 资源点 ID 列表
     */
    int[] getResourceList(int mapId);
}
