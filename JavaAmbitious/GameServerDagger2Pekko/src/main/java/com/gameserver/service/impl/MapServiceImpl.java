package com.gameserver.service.impl;

import com.gameserver.service.MapService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * 地图系统服务实现
 * 
 * 设计说明：
 * 1. 无状态实现，所有计算都基于输入参数
 * 2. 由 Dagger2 管理单例，所有地图 Actor 共享
 * 3. 多个地图 Actor 可以并发调用，无竞态条件
 * 
 * 线程安全保证：
 * - 不维护任何可变状态
 * - 所有计算都是原子操作
 * - 多个线程可以并发调用，无竞态条件
 */
public class MapServiceImpl implements MapService {
    
    private static final Logger logger = LoggerFactory.getLogger(MapServiceImpl.class);

    @Override
    public boolean canMoveTo(int mapId, int x, int y) {
        // 示例：简单的碰撞检测
        // 实际游戏服应该从地图数据查询该位置是否有障碍物
        
        // 假设地图大小为 1000x1000
        if (x < 0 || x > 1000 || y < 0 || y > 1000) {
            return false;  // 超出地图边界
        }
        
        // 假设 (500, 500) 是一个障碍物
        if (x == 500 && y == 500) {
            return false;
        }
        
        logger.debug("[MapService] 检查移动: mapId={}, x={}, y={}, canMove=true", mapId, x, y);
        return true;
    }

    @Override
    public int[] getMonsterList(int mapId) {
        // 示例：返回地图的怪物列表
        // 实际游戏服应该从数据库查询

        switch (mapId) {
            case 1:
                return new int[]{101, 102, 103};  // 地图 1 的怪物
            case 2:
                return new int[]{201, 202, 203, 204};  // 地图 2 的怪物
            default:
                return new int[]{};
        }
    }

    @Override
    public int[] getResourceList(int mapId) {
        // 示例：返回地图的资源点列表
        // 实际游戏服应该从数据库查询

        switch (mapId) {
            case 1:
                return new int[]{1001, 1002};  // 地图 1 的资源点
            case 2:
                return new int[]{2001, 2002, 2003};  // 地图 2 的资源点
            default:
                return new int[]{};
        }
    }
}
