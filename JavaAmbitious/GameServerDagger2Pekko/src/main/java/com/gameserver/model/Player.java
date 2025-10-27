package com.gameserver.model;

/**
 * 玩家数据模型
 * 
 * 设计原则：
 * 1. 所有字段都是 final，确保不可变性
 * 2. 通过 getter 方法访问数据
 * 3. 创建新实例时使用 Builder 模式
 * 
 * 线程安全保证：
 * - 所有字段都是 final，创建后不可修改
 * - 多个 Actor 可以并发读取同一个 Player 对象
 * - 不需要加锁或同步
 */
public final class Player {
    
    private final long id;
    private final String account;
    private final int level;
    private final int experience;
    private final int health;
    private final int mana;

    public Player(long id, String account, int level, int experience, int health, int mana) {
        this.id = id;
        this.account = account;
        this.level = level;
        this.experience = experience;
        this.health = health;
        this.mana = mana;
    }

    // ============ Getter 方法 ============
    
    public long getId() { return id; }
    public String getAccount() { return account; }
    public int getLevel() { return level; }
    public int getExperience() { return experience; }
    public int getHealth() { return health; }
    public int getMana() { return mana; }

    // ============ Builder 模式 ============
    
    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private long id;
        private String account;
        private int level = 1;
        private int experience = 0;
        private int health = 100;
        private int mana = 50;

        public Builder id(long id) {
            this.id = id;
            return this;
        }

        public Builder account(String account) {
            this.account = account;
            return this;
        }

        public Builder level(int level) {
            this.level = level;
            return this;
        }

        public Builder experience(int experience) {
            this.experience = experience;
            return this;
        }

        public Builder health(int health) {
            this.health = health;
            return this;
        }

        public Builder mana(int mana) {
            this.mana = mana;
            return this;
        }

        public Player build() {
            return new Player(id, account, level, experience, health, mana);
        }
    }

    @Override
    public String toString() {
        return "Player{" +
                "id=" + id +
                ", account='" + account + '\'' +
                ", level=" + level +
                ", experience=" + experience +
                ", health=" + health +
                ", mana=" + mana +
                '}';
    }
}
