package com.gameserver.entity;

import com.vladmihalcea.hibernate.type.json.JsonBinaryType;
import org.hibernate.annotations.Type;

import jakarta.persistence.*;
import java.time.LocalDateTime;
import java.util.HashMap;
import java.util.Map;

/**
 * 玩家实体
 * 
 * 设计特点：
 * 1. 使用 JSONB 字段存储动态属性（力量、敏捷、智力等）
 * 2. 使用 JSONB 字段存储装备数据（避免表爆炸）
 * 3. 使用 JSONB 字段存储 Buff 状态
 * 4. 与 Dagger2 注入的 DAO 配合使用
 * 5. 支持 PostgreSQL 特有类型
 */
@Entity
@Table(name = "players", indexes = {
    @Index(name = "idx_account", columnList = "account"),
    @Index(name = "idx_level", columnList = "level"),
    @Index(name = "idx_experience", columnList = "experience")
})
public class Player {
    
    @Id
    private Long id;
    
    @Column(nullable = false, unique = true)
    private String account;
    
    @Column(nullable = false)
    private Integer level = 1;
    
    @Column(nullable = false)
    private Long experience = 0L;
    
    @Column(nullable = false)
    private Integer health = 100;
    
    @Column(nullable = false)
    private Integer mana = 50;
    
    /**
     * JSONB 字段：存储动态属性
     * 
     * 示例：
     * {
     *   "strength": 50,
     *   "agility": 45,
     *   "intelligence": 40
     * }
     */
    @Type(JsonBinaryType.class)
    @Column(columnDefinition = "jsonb", nullable = false)
    private Map<String, Object> attributes = new HashMap<>();
    
    /**
     * JSONB 字段：存储装备数据
     * 
     * 示例：
     * {
     *   "weapon_id": 1001,
     *   "armor_id": 2001,
     *   "accessory_id": 3001
     * }
     */
    @Type(JsonBinaryType.class)
    @Column(columnDefinition = "jsonb", nullable = false)
    private Map<String, Object> equipment = new HashMap<>();
    
    /**
     * JSONB 字段：存储 Buff 状态
     * 
     * 示例：
     * {
     *   "strength_buff": {"value": 10, "duration": 300},
     *   "speed_buff": {"value": 20, "duration": 600}
     * }
     */
    @Type(JsonBinaryType.class)
    @Column(columnDefinition = "jsonb", nullable = false)
    private Map<String, Object> buffs = new HashMap<>();
    
    @Column(nullable = false, updatable = false)
    private LocalDateTime createdAt = LocalDateTime.now();
    
    @Column(nullable = false)
    private LocalDateTime updatedAt = LocalDateTime.now();
    
    // ==================== Getters and Setters ====================
    
    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }
    
    public String getAccount() { return account; }
    public void setAccount(String account) { this.account = account; }
    
    public Integer getLevel() { return level; }
    public void setLevel(Integer level) { this.level = level; }
    
    public Long getExperience() { return experience; }
    public void setExperience(Long experience) { this.experience = experience; }
    
    public Integer getHealth() { return health; }
    public void setHealth(Integer health) { this.health = health; }
    
    public Integer getMana() { return mana; }
    public void setMana(Integer mana) { this.mana = mana; }
    
    public Map<String, Object> getAttributes() { return attributes; }
    public void setAttributes(Map<String, Object> attributes) { this.attributes = attributes; }
    
    public Map<String, Object> getEquipment() { return equipment; }
    public void setEquipment(Map<String, Object> equipment) { this.equipment = equipment; }
    
    public Map<String, Object> getBuffs() { return buffs; }
    public void setBuffs(Map<String, Object> buffs) { this.buffs = buffs; }
    
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }
    public void setUpdatedAt(LocalDateTime updatedAt) { this.updatedAt = updatedAt; }
    
    // ==================== 业务方法 ====================
    
    /**
     * 增加经验值
     */
    public void addExperience(long amount) {
        this.experience += amount;
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 减少血量
     */
    public void reduceHealth(int amount) {
        this.health = Math.max(0, this.health - amount);
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 恢复血量
     */
    public void restoreHealth(int amount) {
        this.health = Math.min(100, this.health + amount);
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 获取属性值
     */
    public int getAttribute(String name) {
        Object value = attributes.get(name);
        return value instanceof Number ? ((Number) value).intValue() : 0;
    }
    
    /**
     * 设置属性值
     */
    public void setAttribute(String name, int value) {
        attributes.put(name, value);
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 检查是否存活
     */
    public boolean isAlive() {
        return health > 0;
    }
    
    /**
     * 获取装备 ID
     */
    public Long getEquipmentId(String slot) {
        Object value = equipment.get(slot);
        return value instanceof Number ? ((Number) value).longValue() : null;
    }
    
    /**
     * 装备物品
     */
    public void equip(String slot, long itemId) {
        equipment.put(slot, itemId);
        this.updatedAt = LocalDateTime.now();
    }
    
    /**
     * 转换为数据模型对象
     * 将实体对象转换为用于消息传递的不可变模型对象
     * 
     * @return 转换后的Player模型对象
     */
    public com.gameserver.model.Player toModel() {
        return com.gameserver.model.Player.builder()
                .id(this.id)
                .account(this.account)
                .level(this.level)
                .experience(this.experience.intValue()) // 注意：entity中的experience是Long，model中是int
                .health(this.health)
                .mana(this.mana)
                .build();
    }
}