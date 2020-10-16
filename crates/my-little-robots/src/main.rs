use mlr::Player;
use mlr::PlayerId;
use mlr::RunnerInput;
use mlr::{RunnerOutput, World};

mod lib;

fn player_run(_input: RunnerInput) -> RunnerOutput {
    println!("Hoi Wereld");
    Ok(Vec::new())
}

fn main() {
    async_std::task::block_on(async {
        // Create the world
        let mut world = World {};

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
    });
}
