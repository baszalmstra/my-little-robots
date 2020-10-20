#!/usr/bin/python3
import sys
import json
from typing import List, Any
from enum import Enum


class TileType(Enum):
    WALL = "wall"
    FLOOR = "floor"
    EXIT = "exit"

    @staticmethod
    def from_json(json: Any):
        t = json["type"]
        if t == "wall":
            return TileType.WALL
        elif t == "floor":
            return TileType.FLOOR
        elif t == "exit":
            return TileType.EXIT
        else:
            return TileType.WALL


class Coord:
    """Coordinate in the world"""

    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    @staticmethod
    def from_json(json: Any):
        return Coord(json[0], json[1])

    def __repr__(self):
        return f"Coord({self.x}, {self.y})"


class Tile:
    def __init__(self, tile_type: TileType, coord: Coord):
        self.tile_type = tile_type
        self.coord = coord

    def __repr__(self):
        return f"Tile({self.tile_type}, {self.coord})"

    @staticmethod
    def from_json(json: Any):
        return Tile(TileType.from_json(json), Coord.from_json(json["coord"]))


class Unit:
    """A unit in the world corresponding to a player"""

    def __init__(self, id: int, player: int, location: Coord):
        self.id = id
        self.player = player
        self.location = location

    @staticmethod
    def from_json(json: Any):
        return Unit(json["id"], json["player"], json["location"])

    def __repr__(self):
        return f"Unit(id={self.id}, player={self.id}, location={self.location})"


class PlayerWorld:
    """The entire world that the player knows"""

    def __init__(self, units: List[Unit], tiles: List[TileType]):
        self.units = units
        self.tiles = tiles

    def __repr__(self):
        return f"PlayerWorld(units={self.units}, tiles={self.tiles})"

    @staticmethod
    def from_json(json: Any):
        units = []
        for u in json["units"]:
            units.append(Unit(u["id"], u["player"], Coord.from_json(u["location"])))
        tiles = []
        for t in json["tiles"]:
            tiles.append(Tile.from_json(t))
        return PlayerWorld(units, tiles)


class PlayerInput:
    """The input that the player receives"""

    def __init__(
        self, player_id: int, turn: int, player_world: PlayerWorld, memory: Any
    ):
        self.player_id = player_id
        self.turn = turn
        self.player_world = player_world
        self.memory = memory

    def __repr__(self):
        return f"PlayerInput(player_id={self.player_id}, turn={self.turn}, player_world={self.player_world}, memory={self.memory})"

    @staticmethod
    def from_json(json: Any):
        return PlayerInput(
            json["player_id"],
            json["turn"],
            PlayerWorld.from_json(json["world"]),
            json["memory"],
        )


def tick(data: PlayerInput):
    """Do the processing here"""
    print(data)


def from_json(json: Any) -> PlayerInput:
    """Create the structures from json"""
    return PlayerInput.from_json(json)


def main():
    # Read json from stdin
    for line in sys.stdin:
        # Process json
        loaded = json.loads(line)
        print(loaded)
        tick(from_json(loaded))
        break


if __name__ == "__main__":
    # Run the main
    main()
