package com.gameserver.actor;

import com.gameserver.model.Player;

/**
 * 玩家 Actor 消息定义
 * 
 * 设计原则：
 * 1. 所有消息都是不可变的（所有字段都是 final）
 * 2. 消息通过 Actor 的消息队列传递
 * 3. 确保线程安全的消息传递
 * 
 * 游戏服最佳实践：
 * - 消息应该包含完整的操作信息
 * - 避免在消息中传递可变对象
 * - 使用 sealed class 或 interface 来定义消息类型
 */
public class PlayerMessage {
    
    /**
     * 玩家登录消息
     */
    public static final class Login {
        public final String account;
        public final String password;

        public Login(String account, String password) {
            this.account = account;
            this.password = password;
        }

        @Override
        public String toString() {
            return "Login{" +
                    "account='" + account + '\'' +
                    '}';
        }
    }

    /**
     * 玩家登录响应
     */
    public static final class LoginResponse {
        public final boolean success;
        public final Player playerData;

        public LoginResponse(boolean success, Player playerData) {
            this.success = success;
            this.playerData = playerData;
        }

        @Override
        public String toString() {
            return "LoginResponse{" +
                    "success=" + success +
                    ", playerData=" + playerData +
                    '}';
        }
    }

    /**
     * 玩家攻击消息
     */
    public static final class Attack {
        public final long targetId;
        public final int skillId;

        public Attack(long targetId, int skillId) {
            this.targetId = targetId;
            this.skillId = skillId;
        }

        @Override
        public String toString() {
            return "Attack{" +
                    "targetId=" + targetId +
                    ", skillId=" + skillId +
                    '}';
        }
    }

    /**
     * 玩家攻击响应
     */
    public static final class AttackResponse {
        public final int damage;

        public AttackResponse(int damage) {
            this.damage = damage;
        }

        @Override
        public String toString() {
            return "AttackResponse{" +
                    "damage=" + damage +
                    '}';
        }
    }

    /**
     * 玩家增加经验消息
     */
    public static final class AddExperience {
        public final int experience;

        public AddExperience(int experience) {
            this.experience = experience;
        }

        @Override
        public String toString() {
            return "AddExperience{" +
                    "experience=" + experience +
                    '}';
        }
    }

    /**
     * 玩家增加经验响应
     */
    public static final class AddExperienceResponse {
        public final int newLevel;

        public AddExperienceResponse(int newLevel) {
            this.newLevel = newLevel;
        }

        @Override
        public String toString() {
            return "AddExperienceResponse{" +
                    "newLevel=" + newLevel +
                    '}';
        }
    }

    /**
     * 玩家登出消息
     */
    public static final class Logout {
        @Override
        public String toString() {
            return "Logout{}";
        }
    }
}
