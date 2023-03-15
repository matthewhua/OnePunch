package io.matt.netty.websocket;


import io.netty.channel.Channel;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.RequestMapping;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Map;

@RequestMapping("index")
@Controller
public class WebsocketController {
    /**
     * @param id     用户主键
     * @param idList 要把消息发送给其他用户的主键
     */
    @RequestMapping("hello1")
    private void hello(Long id, List<Long> idList) {
        //获取所有连接的客户端,如果是集群环境使用redis的hash数据类型存储即可
        Map<String, Channel> channelMap = WebSocketHandler.getChannelMap();
        //获取与用户主键绑定的channel,如果是集群环境使用redis的hash数据类型存储即可
        Map<String, Long> clientMap = WebSocketHandler.getClientMap();
        //解决问题六,websocket集群中一个客户端向其他客户端主动发送消息，如何实现？
        clientMap.forEach((k, v) -> {
            if (idList.contains(v)) {
                Channel channel = channelMap.get(k);
                channel.eventLoop().execute(() -> channel.writeAndFlush(new TextWebSocketFrame(Thread.currentThread().getName() + "服务器时间" + LocalDateTime.now() + "wdy")));
            }
        });
    }

}
