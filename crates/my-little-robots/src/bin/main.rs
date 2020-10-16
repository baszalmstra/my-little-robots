use mlr::RunnerInput;
use mlr::{Coord, Direction, PlayerId};
use mlr::{Player, PlayerAction};
use mlr::{RunnerOutput, Unit, World};

fn player_run(input: RunnerInput) -> RunnerOutput {
    println!("Hoi Wereld");

    // Get all units
    let (my_units, _other_units): (Vec<&Unit>, Vec<&Unit>) = input
        .units
        .iter()
        .partition(|u| u.player == input.player_id);

    // Move all units
    let mut actions = Vec::new();
    for unit in my_units {
        actions.push(PlayerAction::Move(unit.id, Direction::Left));
    }

    Ok(actions)
}

fn main() {
    env_logger::init();

    async_std::task::block_on(async {
        // Create the world
        let mut world = World::new();

        // Create a player to run
        let player = Player {
            id: PlayerId(0),
            runner: Box::new(player_run),
        };

        let mut players = [player];

        // Spawn a unit for every player
        for (i, player) in players.iter().enumerate() {
            world.spawn(
                player.id,
                Coord {
                    x: 10 + i as isize * 10,
                    y: 10,
                },
            );
        }

        // Run the turn in a loop
        loop {
            world = mlr::turn(&mut players, world).await;
            async_std::task::sleep(std::time::Duration::from_millis(500)).await;
        }
    });
}
