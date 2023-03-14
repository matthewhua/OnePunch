package io.matt.behavior.mediator.chatRoom;

public class Client {

    public static void main(String[] args) {
        ChatRoomImpl chatRoom = new ChatRoomImpl();
        ChatUser user1 = new ChatUser(chatRoom, "1", "Spike");
        ChatUser user2 = new ChatUser(chatRoom, "2", "Matthew");
        ChatUser user3 = new ChatUser(chatRoom, "3", "HUO");
        ChatUser user4 = new ChatUser(chatRoom, "4", "HUA");

        chatRoom.addUser(user1);
        chatRoom.addUser(user2);
        chatRoom.addUser(user3);
        chatRoom.addUser(user4);
        user1.send("Hello man", "2");
        user2.send("HEY", "3");
        user3.send("fuck you", "1");
    }
}
