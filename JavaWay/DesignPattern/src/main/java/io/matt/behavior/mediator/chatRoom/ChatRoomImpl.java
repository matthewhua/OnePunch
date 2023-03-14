package io.matt.behavior.mediator.chatRoom;

import java.util.HashMap;
import java.util.Map;

public class ChatRoomImpl implements ChatRoom {

    private Map<String, User> userMap = new HashMap<>();

    @Override
    public void sendMessage(String msg, String userId) {
        User user = userMap.get(userId);
        if (user != null) {
            user.receive(msg);
        }
    }

    @Override
    public void addUser(User user) {
        this.userMap.put(user.getId(), user);
    }
}
