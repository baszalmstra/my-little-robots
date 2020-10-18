#!/usr/bin/python3
import sys
import json
from typing import List, Any


class Coord:
    """Coordinate in the world"""
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    @staticmethod
    def from_json(json: Any):
        return Coord(json["x"], json["y"])

    def __repr__(self):
        return f"Coord({self.x}, {self.y})"


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
    def __init__(self, units: List[Unit]):
        self.units = units

    def __repr__(self):
        return f"PlayerWorld(units={self.units})"

    @staticmethod
    def from_json(json: Any):
        units = []
        for u in json["units"]:
            units.append(
                Unit(u["unit_id"], u["player_id"], Coord.from_json(u["location"]))
            )
        return PlayerWorld(units)


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
        PlayerInput(
            json["player_id"],
            json["turn"],
            PlayerWorld.from_json(json["world"]),
            json["memory"],
        )


def process(data: object):
    print(data)


def main():
    # Read json from stdin
    for line in sys.stdin:
        # Process json
        process(json.loads(line))
        break


if __name__ == "__main__":
    # Run the main
    main()
