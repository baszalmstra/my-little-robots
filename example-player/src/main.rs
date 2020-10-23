use mlr_api::{
    Coord, Direction, PlayerAction, PlayerInput, PlayerOutput, TileType, Unit, UnitId, API_VERSION,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize)]
struct Memory {
    #[serde(default)]
    directions: HashMap<UnitId, Direction>,

    #[serde(default)]
    walls: HashSet<Coord>,
}

/// Returns the direction right from the current direction
fn right(direction: Direction) -> Direction {
    match direction {
        Direction::Left => Direction::Up,
        Direction::Right => Direction::Down,
        Direction::Up => Direction::Right,
        Direction::Down => Direction::Left,
    }
}

/// Returns the direction left from the current direction
fn left(direction: Direction) -> Direction {
    match direction {
        Direction::Left => Direction::Down,
        Direction::Right => Direction::Up,
        Direction::Up => Direction::Left,
        Direction::Down => Direction::Right,
    }
}

/// This function is called every tick. It should return actions for all the units that the player
/// owns.
fn tick(input: PlayerInput<Memory>) -> PlayerOutput<Memory> {
    let PlayerInput {
        version,
        world,
        mut memory,
        player_id,
        turn: _,
    } = input;

    assert_eq!(version, API_VERSION, "mismatched api version");

    let mut rng = rand::thread_rng();

    // Store vision
    for coord in world.tiles.iter() {
        if coord.tile_type == TileType::Wall {
            memory.walls.insert(coord.coord);
        }
    }

    // Get all units
    let (my_units, _other_units): (Vec<&Unit>, Vec<&Unit>) =
        world.units.iter().partition(|u| u.player == player_id);

    // Move all units
    let mut actions = Vec::new();
    for unit in my_units {
        // Get the direction this unit took last time
        let current_direction = memory
            .directions
            .get(&unit.id)
            .copied()
            .unwrap_or_else(|| Direction::random(&mut rng));

        // We always want to go right
        let mut direction = right(current_direction);

        // Check if thats possible, otherwise, face to the left and try again
        let direction = loop {
            let new_pos = unit.location + direction;
            if new_pos.x > 0 && new_pos.y > 0 && !memory.walls.contains(&new_pos) {
                break direction;
            } else {
                direction = left(direction);
            }
        };

        // Store the direction we're going to take
        memory.directions.insert(unit.id, direction);

        // Perform the move
        actions.push(PlayerAction::Move {
            unit: unit.id,
            direction,
        });
    }

    PlayerOutput { actions, memory }
}

fn main() {
    let mut str = String::new();
    std::io::stdin()
        .read_line(&mut str)
        .expect("could not read input");

    let output =
        tick(serde_json::from_str::<PlayerInput<Memory>>(&str).expect("could not convert input"));

    let output_str = serde_json::to_string(&output).unwrap();
    println!("__mlr_output:{}", output_str);
}
