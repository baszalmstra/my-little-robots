use crate::{Player, PlayerRunner, World, GameState};
use async_std::sync::{Sender};
use mlr_api::{PlayerId, Coord};
use std::time::Duration;
use serde_json::json;

/// A `Battle` is a struct that contains information about a battle to be played
pub struct Battle {
    players: Vec<Box<dyn PlayerRunner>>,
}

impl Default for Battle {
    fn default() -> Self {
        Battle {
            players: Default::default(),
        }
    }
}

impl Battle {
    /// Adds a player to the battle
    pub fn add_player(&mut self, player: Box<dyn PlayerRunner>) -> PlayerId {
        let player_id = PlayerId(self.players.len());
        self.players.push(player);
        player_id
    }
}

impl Battle {
    /// Runs the battle to completion, returns the winning player.
    pub async fn run(self, tick_duration: Option<Duration>, tick_update: Option<Sender<World>>) -> PlayerId {
        let players = self
            .players
            .into_iter()
            .enumerate()
            .map(|(i, runner)| Player {
                id: PlayerId(i),
                runner,
                memory: json!({}),
            })
            .collect::<Vec<_>>();

        let mut game_state = GameState {
            players,
            world: World::default(),
        };

        // Spawn a unit for every player
        for (i, player) in game_state.players.iter().enumerate() {
            game_state
                .world
                .spawn_unit(player.id, Coord::new(10 + i as isize * 10, 10));
        }

        // Run the turn in a loop
        loop {
            game_state = game_state.turn().await;
            if let Some(sender) = &tick_update {
                sender.send(game_state.world.clone()).await
            }
            if let Some(unit) = game_state.world.units_on_exits().next() {
                break unit.player;
            }
            if let Some(duration) = &tick_duration {
                async_std::task::sleep(*duration).await;
            }
        }
    }
}
