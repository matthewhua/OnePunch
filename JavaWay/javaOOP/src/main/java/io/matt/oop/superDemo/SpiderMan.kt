package io.matt.oop.superDemo


/**
 * @author Matthew Hua
 * @Date 2023/6/9
 */
class SpiderMan(val player: Creature) : CommonPlayer(player) {

    constructor() : this(Adminster.creature)
}