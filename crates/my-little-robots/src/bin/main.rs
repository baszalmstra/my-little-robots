use mlr::application;
use mlr::PlayerId;
use mlr::RunnerInput;
use mlr::{Player, PlayerAction};
use mlr::{RunnerOutput, World};
use std::sync::mpsc::channel;

fn player_run(_input: RunnerInput) -> RunnerOutput {
    println!("Hoi Wereld");
    Ok(vec![PlayerAction::DoNothing])
}

fn main() {
    env_logger::init();

    // Create the world
    let mut world = World::new();
    let world_clone = world.clone();

    let (_sender, reciever) = channel();

    std::thread::spawn(|| {
        async_std::task::block_on(async move {
            // Create a player to run
            let player = Player {
                id: PlayerId(0),
                runner: Box::new(player_run),
            };

            let mut players = [player];

            // Run the turn in a loop
            loop {
                world = mlr::turn(&mut players, world).await;
                async_std::task::sleep(std::time::Duration::from_millis(500)).await;
            }
        })
    });

    // Render our world
    application::run(world_clone, reciever).expect("Error while rendering");
}
