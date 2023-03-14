package io.matt.behavior.mediator.chatRoom;

public interface ChatRoom {
    void sendMessage(String msg, String userId);

    void addUser(User user);
}
