use mlr::application;
use mlr::PlayerId;
use mlr::RunnerInput;
use mlr::World;
use mlr::{Coord, Direction};
use mlr::{Player, PlayerAction};
use mlr::{RunnerOutput, Unit};
use std::sync::mpsc::channel;

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

    // Create the world
    let mut world = World::new();
    let world_clone = world.clone();

    let (sender, reciever) = channel();

    std::thread::spawn(|| {
        async_std::task::block_on(async move {
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
                sender
                    .send(world.clone())
                    .expect("Could not send updated map");
                async_std::task::sleep(std::time::Duration::from_millis(500)).await;
            }
        });
    });

    // Render our world
    application::run(world_clone, reciever).expect("Error while rendering");
}
