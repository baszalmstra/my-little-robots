use mlr_api::{Direction, PlayerAction, PlayerInput, PlayerOutput, Unit};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Memory {
    #[serde(default)]
    turn: usize,
}

/// This function is called every tick. It should return actions for all the units that the player
/// owns.
fn tick(input: PlayerInput<Memory>) -> PlayerOutput<Memory> {
    let PlayerInput {
        world,
        mut memory,
        player_id,
        turn,
    } = input;

    memory.turn = turn;

    // Get all units
    let (my_units, _other_units): (Vec<&Unit>, Vec<&Unit>) =
        world.units.iter().partition(|u| u.player == player_id);

    // Move all units
    let mut actions = Vec::new();
    for unit in my_units {
        actions.push(PlayerAction::Move(unit.id, Direction::Left));
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
