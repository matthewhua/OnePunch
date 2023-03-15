package io.matt.netty.websocket;

import io.netty.channel.ChannelHandlerContext;

public class WebsocketRunnable implements Runnable {

    private ChannelHandlerContext context;

    private MessageRequest messageRequest;

    public WebsocketRunnable(ChannelHandlerContext ctx, MessageRequest messageRequest) {
        this.context = ctx;
        this.messageRequest = messageRequest;
    }


    @Override
    public void run() {
        System.out.printf("%s 正在发送 %s", context, messageRequest);
    }

}
