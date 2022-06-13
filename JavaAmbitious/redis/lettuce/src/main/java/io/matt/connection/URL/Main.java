package io.matt.connection.URL;

import io.lettuce.core.RedisClient;
import io.lettuce.core.api.StatefulRedisConnection;

/**
 * @author Matthew
 * @date 2022/6/1
 */
public class Main {

    public static String host = "redis.zygg.com";

    public static Integer port = 6379;

    public static void main(String[] args) {

        // Syntax: redis://[password@]host[:port][/databaseNumber]
        // Syntax: redis://[username:password@]host[:port][/databaseNumber]
        RedisClient redisClient = RedisClient.create("redis://@redis.zygg.com:6379/0");
        StatefulRedisConnection<String, String> connection = redisClient.connect();

        System.out.println("Connected to Redis");

        connection.close();
        redisClient.shutdown();
    }
}
