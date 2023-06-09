package io.matt.oop.superDemo;

import org.jetbrains.annotations.NotNull;

/**
 * @author Matthew Hua
 * @Date 2023/6/9
 */
public class Worker extends CommonPlayer {

    @NotNull
    final Creature player;

    public @NotNull Creature getPlayer() {
        return player;
    }


    public Worker(@NotNull Creature player) {
        super(player);
        this.player = player;
    }


    public Worker() {
        super(Adminster.creature);
        this.player = Adminster.creature;
    }

}
