package com.gameserver.dao;

import com.gameserver.entity.Backpack;
import java.util.Optional;
import java.util.List;

/**
 * 背包 DAO 接口
 * 
 * 职责：
 * 1. 背包物品的 CRUD 操作
 * 2. 物品增减的事务性操作
 * 3. 玩家背包查询
 */
public interface BackpackDAO {
    
    /**
     * 根据玩家和物品 ID 查找背包记录
     */
    Optional<Backpack> findByPlayerAndItem(Long playerId, Long itemId);
    
    /**
     * 查询玩家的所有背包物品
     */
    List<Backpack> findByPlayerId(Long playerId);
    
    /**
     * 保存背包记录
     */
    void save(Backpack backpack);
    
    /**
     * 删除背包记录
     */
    void delete(Long backpackId);
    
    /**
     * 删除玩家的特定物品
     */
    void deleteByPlayerAndItem(Long playerId, Long itemId);
}
