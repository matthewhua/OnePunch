package io.matt.oop.superDemo;

/**
 * @author Matthew Hua
 * @Date 2023/6/9
 */
public class Adminster extends CommonPlayer {

   static Creature creature = new Creature() {
        @Override
        public String getPlayerId() {
            return "Mathew";
        }

        @Override
        public void setPlayerId(String name) {

        }
    };


    public Adminster(Creature name) {
        super(name);
    }

    public Adminster() {
        super(Adminster.creature);
    }

}
