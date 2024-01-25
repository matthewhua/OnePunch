package com.matt.springwebflux;

import org.springframework.web.reactive.socket.WebSocketMessage;
import org.springframework.web.reactive.socket.client.ReactorNettyWebSocketClient;
import reactor.core.publisher.Flux;

import java.net.URI;
import java.time.Duration;

public class WSClient {

  public static void main(String[] args) {
    ReactorNettyWebSocketClient client = new ReactorNettyWebSocketClient();
    client.execute(URI.create("ws://localhost:8080/echo"), session ->
                    session.send(Flux.just(session.textMessage("你好")))
                            .thenMany(session.receive().take(1).map(WebSocketMessage::getPayloadAsText))
                            .doOnNext(System.out::println)
                            .then())
            .block(Duration.ofMillis(5000));
  }
}
