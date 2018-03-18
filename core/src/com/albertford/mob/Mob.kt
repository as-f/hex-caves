package com.albertford.mob

import com.albertford.util.Direction
import com.albertford.Level
import com.albertford.util.Pos
import com.albertford.Tile

interface Mob {
    var pos: Pos
    var facingRight: Boolean
    var lastMove: Direction?

    fun canMove(targetTile: Tile) {

    }

    fun move(level: Level, direction: Direction): Boolean {
        facingRight = when (direction) {
            Direction.EAST, Direction.NORTHEAST, Direction.SOUTHEAST -> true
            else -> false
        }
        val targetTile = level.tiles[pos + direction]
        if (!targetTile.terrain.passable || targetTile.mob != null) {
            return false
        }
        level.tiles[pos].mob = null
        pos += direction
        level.tiles[pos].mob = this
        lastMove = direction
        return true
    }
}

class Player : Mob {
    override var pos = Pos(0, 0)
    override var facingRight = false
    override var lastMove: Direction? = null
    var hasKey = false
    var sneaky = true
}
