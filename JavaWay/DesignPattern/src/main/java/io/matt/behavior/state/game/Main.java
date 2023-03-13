package io.matt.behavior.state.game;

public class Main {

    public static void main(String[] args) {
        GameAccount jungle = new GameAccount("Jungle");

        for (int i = 0; i < 5; i++) {
            System.out.printf("%d \n", i + 1);
            jungle.playCard();
        }


    }
}
