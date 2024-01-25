package com.matt.springwebflux;

import org.springframework.web.reactive.function.server.ServerRequest;
import org.springframework.web.reactive.function.server.ServerResponse;
import reactor.core.publisher.Mono;

public class CityHandler {

  public Mono<ServerResponse> helloCity(ServerRequest request) {
    return ServerResponse.ok().body(Mono.just("Hello, City!"), String.class);
  }

}
