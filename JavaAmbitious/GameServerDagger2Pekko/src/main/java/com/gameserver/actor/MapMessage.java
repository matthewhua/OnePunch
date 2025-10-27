package com.gameserver.actor;

/**
 * 地图 Actor 消息定义
 * 
 * 设计原则：
 * 1. 所有消息都是不可变的（所有字段都是 final）
 * 2. 消息通过 Actor 的消息队列传递
 * 3. 确保线程安全的消息传递
 */
public class MapMessage {
    
    /**
     * 玩家进入地图消息
     */
    public static final class PlayerEnter {
        public final long playerId;

        public PlayerEnter(long playerId) {
            this.playerId = playerId;
        }

        @Override
        public String toString() {
            return "PlayerEnter{" +
                    "playerId=" + playerId +
                    '}';
        }
    }

    /**
     * 玩家进入地图响应
     */
    public static final class PlayerEnterResponse {
        public final boolean success;

        public PlayerEnterResponse(boolean success) {
            this.success = success;
        }

        @Override
        public String toString() {
            return "PlayerEnterResponse{" +
                    "success=" + success +
                    '}';
        }
    }

    /**
     * 玩家离开地图消息
     */
    public static final class PlayerLeave {
        public final long playerId;

        public PlayerLeave(long playerId) {
            this.playerId = playerId;
        }

        @Override
        public String toString() {
            return "PlayerLeave{" +
                    "playerId=" + playerId +
                    '}';
        }
    }

    /**
     * 玩家离开地图响应
     */
    public static final class PlayerLeaveResponse {
        public final boolean success;

        public PlayerLeaveResponse(boolean success) {
            this.success = success;
        }

        @Override
        public String toString() {
            return "PlayerLeaveResponse{" +
                    "success=" + success +
                    '}';
        }
    }

    /**
     * 获取地图玩家数量消息
     */
    public static final class GetPlayerCount {
        @Override
        public String toString() {
            return "GetPlayerCount{}";
        }
    }

    /**
     * 获取地图玩家数量响应
     */
    public static final class GetPlayerCountResponse {
        public final int playerCount;

        public GetPlayerCountResponse(int playerCount) {
            this.playerCount = playerCount;
        }

        @Override
        public String toString() {
            return "GetPlayerCountResponse{" +
                    "playerCount=" + playerCount +
                    '}';
        }
    }
}
