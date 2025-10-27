package com.gameserver.entity;

import jakarta.persistence.*;
import java.time.LocalDateTime;

/**
 * 背包实体
 * 
 * 设计特点：
 * 1. 与 Player 一对多关系
 * 2. 支持物品堆叠（quantity 字段）
 * 3. 与 ORM 配合实现背包操作的事务性
 * 4. 支持物品增减操作
 */
@Entity
@Table(name = "backpack", indexes = {
    @Index(name = "idx_player_item", columnList = "player_id, item_id", unique = true)
})
public class Backpack {
    
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(nullable = false)
    private Long playerId;
    
    @Column(nullable = false)
    private Long itemId;
    
    @Column(nullable = false)
    private Integer quantity = 1;
    
    @Column(nullable = false, updatable = false)
    private LocalDateTime createdAt = LocalDateTime.now();
    
    @Column(nullable = false)
    private LocalDateTime updatedAt = LocalDateTime.now();
    
    // ==================== Getters and Setters ====================
    
    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }
    
    public Long getPlayerId() { return playerId; }
    public void setPlayerId(Long playerId) { this.playerId = playerId; }
    
    public Long getItemId() { return itemId; }
    public void setItemId(Long itemId) { this.itemId = itemId; }
    
    public Integer getQuantity() { return quantity; }
    public void setQuantity(Integer quantity) { this.quantity = quantity; }
    
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }
    public void setUpdatedAt(LocalDateTime updatedAt) { this.updatedAt = updatedAt; }
    
    // ==================== 业务方法 ====================
    
    /**
     * 增加物品数量
     */
    public void addQuantity(int amount) {
        this.quantity += amount;
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 减少物品数量
     * 
     * @return true 如果成功减少，false 如果物品不足
     */
    public boolean reduceQuantity(int amount) {
        if (this.quantity < amount) {
            return false;  // 物品不足
        }
        this.quantity -= amount;
        this.updatedAt = LocalDateTime.now();
        return true;
    }
    
    /**
     * 检查物品是否足够
     */
    public boolean hasEnough(int amount) {
        return this.quantity >= amount;
    }
}
