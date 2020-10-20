#!/usr/bin/python3
from mlr_py import api
import random


def turn(input: api.PlayerInput) -> api.PlayerOutput:
    """Do the processing here"""
    all_directions = [
        api.Direction.LEFT,
        api.Direction.RIGHT,
        api.Direction.DOWN,
        api.Direction.UP,
    ]

    # Choose a random direction for now
    actions = [
        api.PlayerAction(u.id, random.choice(all_directions))
        for u in input.get_my_units()
    ]

    return api.PlayerOutput(actions)


if __name__ == "__main__":
    # Run the main
    api.do_turn(turn_function=turn)
