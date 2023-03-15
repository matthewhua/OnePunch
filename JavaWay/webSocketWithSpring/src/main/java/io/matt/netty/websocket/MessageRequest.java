package io.matt.netty.websocket;


import lombok.Data;

import java.io.Serializable;

@Data
public class MessageRequest implements Serializable {

    private Long unionId;

    private Integer current = 1;

    private int size = 10;
}
