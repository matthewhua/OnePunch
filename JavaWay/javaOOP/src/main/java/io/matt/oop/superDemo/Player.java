package io.matt.oop.superDemo;

/**
 * @author Matthew Hua
 * @Date 2023/6/9
 */
public abstract class Player implements Creature {

    String playerId;

    @Override
    public String getPlayerId() {
        return playerId;
    }

    @Override
    public void setPlayerId(String playerId) {
        this.playerId = playerId;
    }

    public Player(Creature creature) {
        this.playerId = creature.getPlayerId();
    }

}
