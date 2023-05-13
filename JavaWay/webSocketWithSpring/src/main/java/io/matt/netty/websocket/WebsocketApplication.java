package io.matt.netty.websocket;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import javax.annotation.Resource;

@Slf4j
@Component
@SpringBootApplication
public class WebsocketApplication {

    @Resource
    private WebsocketInitialization websocketInitialization;

    @PostConstruct
    public void start() {
        try {
            log.info(Thread.currentThread().getName() + ":websocket启动中......");
            websocketInitialization.init();
            log.info(Thread.currentThread().getName() + ":websocket启动成功！！！");
        } catch (Exception e) {
            log.error("websocket发生错误：", e);
        }
    }

    public static void main(String[] args) {
        ConfigurableApplicationContext run = SpringApplication.run(WebsocketApplication.class);
    }
}
