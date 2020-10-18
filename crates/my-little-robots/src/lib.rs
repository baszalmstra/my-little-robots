pub mod application;
pub mod map;
pub mod runner;

use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use self::map::Map;
use crate::map::new_map_prim;
use futures::channel::mpsc::unbounded;
use futures::{SinkExt, StreamExt};
use mlr_api::{Coord, Direction, PlayerAction, PlayerId, PlayerInput, PlayerMemory, PlayerOutput, PlayerWorld, RunnerError, TileType, Unit, UnitId, PlayerTile};
use std::collections::HashSet;
use itertools::Itertools;

/// A `World` defines the state of the world.
#[derive(Clone, Eq, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub struct World {
    map: Map,
    units: Vec<Unit>,
}

impl Default for World {
    fn default() -> World {
        World {
            //map: new_map_test(80, 50),
            map: new_map_prim(80, 50),
            units: Vec::new(),
        }
    }
}

impl World {
    /// Applies the specified `actions` to an instance and returns a modified instance where these
    /// actions have been applied.
    fn apply(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        for action in actions {
            match action {
                Action::Move(unit_id, direction) => {
                    let unit = &mut self.units[unit_id.0];
                    let new_location = unit.location + direction;
                    if self.map.can_enter_tile(new_location) {
                        unit.location = new_location;
                    }
                }
            }
        }
        self
    }

    /// Creates a snapshot of the world as seen by the given Player.
    fn player_world(&self, player_id: PlayerId) -> PlayerWorld {
        let player_units = self
            .units
            .iter()
            .filter(|unit| unit.player == player_id)
            .cloned()
            .collect_vec();

        let tiles = player_units
            .iter()
            .map(|unit| self.map.field_of_view(unit.location, 7))
            .flatten()
            .map(|coord| {
                PlayerTile {
                    coord,
                    tile_type: self.map[coord],
                }
            })
            .collect();

        PlayerWorld {
            units: player_units,
            tiles,
        }
    }

    /// Spawns a unit in the world
    pub fn spawn_unit(&mut self, player: PlayerId, location: Coord) -> UnitId {
        let id = UnitId(self.units.len());
        self.units.push(Unit {
            id,
            player,
            location,
        });
        id
    }

    /// Returns the units that are currently standing on an exit
    pub fn units_on_exits(&self) -> impl Iterator<Item = &Unit> {
        self.units
            .iter()
            .filter(move |unit| self.map[unit.location] == TileType::Exit)
    }
}

/// Describes an action in the world which may have been undertaken by any player
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum Action {
    Move(UnitId, Direction),
}

/// The PlayerRunner can be implemented to produce actions for a current snapshot of the world.
#[async_trait]
pub trait PlayerRunner: Send {
    /// Given the current state of the world, returns the actions that should be executed.
    async fn run(&mut self, input: PlayerInput) -> Result<PlayerOutput, RunnerError>;
}

// Implement `PlayerRunner` for a functions
#[async_trait]
impl<F> PlayerRunner for F
where
    F: FnMut(PlayerInput) -> Result<PlayerOutput, RunnerError> + Send,
{
    async fn run(&mut self, input: PlayerInput) -> Result<PlayerOutput, RunnerError> {
        (self)(input)
    }
}

/// Represents everything of a specific player.
pub struct Player {
    /// The unique id of this player
    pub id: PlayerId,

    /// The function to generate actions from the current state of the world
    pub runner: Box<dyn PlayerRunner>,

    /// The current player memory
    pub memory: PlayerMemory,
}

/// Represents the current game state
pub struct GameState {
    pub players: Vec<Player>,
    pub world: World,
    pub turn: usize,
}

impl GameState {
    pub async fn turn(mut self) -> Self {
        let (action_sender, action_receiver) = unbounded();
        let world_ref = &self.world;
        let turn = self.turn;
        let player_iter_fut = futures::stream::iter(self.players.iter_mut()).for_each_concurrent(
            None,
            move |player| {
                let mut action_sender = action_sender.clone();
                async move {
                    // Construct the input for the player
                    let player_input = PlayerInput {
                        player_id: player.id,
                        turn,
                        world: world_ref.player_world(player.id),
                        memory: player.memory.clone(),
                    };

                    // Run the player runner
                    let player_result = player.runner.run(player_input).await;

                    // Check the output for errors
                    let output = match player_result {
                        Err(err) => {
                            log::error!("Player {:?}: {}", player.id, err);
                            return;
                        }
                        Ok(output) => output,
                    };

                    // Validate all the actions
                    for player_action in output.actions {
                        match validate_action(player_action, player.id, world_ref) {
                            Err(err) => {
                                log::error!("Player {:?}: invalid action: {}", player.id, err);
                            }
                            Ok(action) => {
                                action_sender
                                    .send(action)
                                    .await
                                    .expect("error sending action");
                            }
                        }
                    }

                    // Store the memory of the player
                    player.memory = output.memory;
                }
            },
        );

        let gather_actions_fut = action_receiver.collect::<Vec<_>>();
        let (_, actions) = futures::future::join(player_iter_fut, gather_actions_fut).await;
        self.world = self.world.apply(actions);
        self.turn += 1;

        self
    }
}

/// An error that might occur when a user sends an action that is not possible.
#[derive(Error, Clone, Debug)]
pub enum ActionValidationError {
    #[error("Invalid action")]
    InvalidAction(String),
}

/// Given an action from a player turn it into an action that can be applied to the world. Returns
/// an error if the action cannot be performed by the player.
fn validate_action(
    action: PlayerAction,
    player: PlayerId,
    world: &World,
) -> Result<Action, ActionValidationError> {
    match action {
        PlayerAction::Move(unit, direction) => {
            if world.units[unit.0].player != player {
                Err(ActionValidationError::InvalidAction(
                    "action points to invalid unit".to_string(),
                ))
            } else {
                Ok(Action::Move(unit, direction))
            }
        }
    }
}
