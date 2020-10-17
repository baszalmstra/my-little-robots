use mlr::application;
use mlr::runner::Runner;
use mlr::GameState;
use mlr::Player;
use mlr::{random_direction, World};
use mlr_api::{Coord, PlayerAction, PlayerId, PlayerInput, PlayerOutput, RunnerError, Unit};
use serde_json::json;

fn player_run(input: PlayerInput) -> Result<PlayerOutput, RunnerError> {
    let mut rng = rand::thread_rng();

    // Get all units
    let (my_units, _other_units): (Vec<&Unit>, Vec<&Unit>) = input
        .world
        .units
        .iter()
        .partition(|u| u.player == input.player_id);

    // Move all units
    let mut actions = Vec::new();
    for unit in my_units {
        actions.push(PlayerAction::Move(unit.id, random_direction(&mut rng)));
    }

    Ok(PlayerOutput {
        actions,
        memory: input.memory,
    })
}

fn main() {
    env_logger::init();

    let example_location = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("example-player{}", std::env::consts::EXE_SUFFIX));
    println!(
        "Assuming that the example-player application is located at: {}",
        example_location.display()
    );

    let mut game_state = GameState {
        players: vec![
            Player {
                id: PlayerId(0),
                runner: Box::new(Runner::new_cmd(example_location, vec!["drol"])),
                memory: json!({}),
            },
            Player {
                id: PlayerId(1),
                runner: Box::new(player_run),
                memory: json!({}),
            },
            Player {
                id: PlayerId(2),
                runner: Box::new(player_run),
                memory: json!({}),
            },
            Player {
                id: PlayerId(3),
                runner: Box::new(player_run),
                memory: json!({}),
            },
        ],
        world: World::default(),
        turn: 0,
    };

    // Spawn a unit for every player
    for (i, player) in game_state.players.iter().enumerate() {
        game_state
            .world
            .spawn_unit(player.id, Coord::new(10 + i as isize * 10, 10));
    }

    // Create the world
    let (sender, receiver) = async_watch::channel(game_state.world.clone());

    std::thread::spawn(|| {
        async_std::task::block_on(async move {
            // Run the turn in a loop
            loop {
                game_state = game_state.turn().await;
                if sender.send(game_state.world.clone()).is_err() {
                    break; // Sender closed
                }
                if game_state.world.units_on_exits().next().is_some() {
                    break;
                }
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
    });

    // Render our world
    application::run(receiver).expect("Error while rendering");
}
