package io.matt.connection.redisUrl;

import io.lettuce.core.RedisClient;
import io.lettuce.core.RedisURI;
import io.matt.connection.URL.Main;

import java.util.concurrent.TimeUnit;

/**
 * @author Matthew
 * @date 2022/6/1
 */
public class RedisURLDemo {

    public static void main(String[] args) {

    }



    private static void usingBasicURI(){
        RedisClient redisClient = RedisClient.create("redis://@redis.zygg.com:6379/0");
        redisClient.setDefaultTimeout(20, TimeUnit.SECONDS);
        redisClient.shutdown();
    }

    private static void usingRedisURI(){
        RedisURI build = RedisURI.Builder.redis(Main.host)
                .withPort(Main.port) //可省略
                .withDatabase(8)
                .build();
        ;
    }
}
