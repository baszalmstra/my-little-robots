mod map;
mod unit;

use async_trait::async_trait;
use futures::future::join_all;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use self::map::Map;
pub use self::unit::{Unit, UnitId};
use std::ops::Add;

/// A `PlayerId` uniquely describes a single Player
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PlayerId(pub usize);

/// A `World` defines the state of the world.
#[derive(Clone, Eq, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub struct World {
    map: Map,
    units: Vec<Unit>,
}

/// A coordinate in the world
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Coord {
    pub x: isize,
    pub y: isize,
}

impl Coord {
    pub fn new(x: isize, y: isize) -> Coord {
        Coord { x, y }
    }
}

impl Add<Direction> for Coord {
    type Output = Coord;

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Left => Coord::new(self.x - 1, self.y),
            Direction::Right => Coord::new(self.x + 1, self.y),
            Direction::Top => Coord::new(self.x, self.y - 1),
            Direction::Bottom => Coord::new(self.x, self.y + 1),
        }
    }
}

/// A direction
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

impl World {
    pub fn new() -> World {
        World {
            map: Map::new(80, 50),
            units: Vec::new(),
        }
    }

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
        PlayerWorld {
            player_id,
            units: self
                .units
                .iter()
                .filter(|unit| unit.player == player_id)
                .cloned()
                .collect(),
        }
    }

    /// Spawns a unit in the world
    pub fn spawn(&mut self, player: PlayerId, location: Coord) -> UnitId {
        let id = UnitId(self.units.len());
        self.units.push(Unit {
            id,
            player,
            location,
        });
        id
    }
}

/// A `PlayerWorld` represents only the visible parts of a world for a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerWorld {
    pub player_id: PlayerId,
    pub units: Vec<Unit>,
}

/// Describes a possible action that can be performed in the world as ordered by a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum PlayerAction {
    Move(UnitId, Direction),
}

/// Describes an action in the world which may have been undertaken by any player
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum Action {
    Move(UnitId, Direction),
}

/// The PlayerRunner can be implemented to produce actions for a current snapshot of the world.
#[async_trait]
pub trait PlayerRunner {
    /// Given the current state of the world, returns the actions that should be executed.
    async fn run(&mut self, input: RunnerInput) -> RunnerOutput;
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum RunnerError {
    #[error("internal error")]
    InternalError,

    #[error("the program exited before it returned any data")]
    NoData,

    #[error("the program took too long, past the time limit of {0:?}")]
    Timeout(Duration),
}

/// The input for a `PlayerRunner`
pub type RunnerInput = PlayerWorld;

/// The output of a `PlayerRunner`
pub type RunnerOutput = Result<Vec<PlayerAction>, RunnerError>;

// Implement `PlayerRunner` for a functions
#[async_trait]
impl<F> PlayerRunner for F
where
    F: FnMut(RunnerInput) -> RunnerOutput + Send,
{
    async fn run(&mut self, input: RunnerInput) -> RunnerOutput {
        (self)(input)
    }
}

/// Represents everything of a specific player.
pub struct Player {
    pub id: PlayerId,
    pub runner: Box<dyn PlayerRunner>,
}

/// Runs a single turn on the world
pub async fn turn(players: &mut [Player], world: World) -> World {
    // Get the actions from all the players
    let actions = join_all(players.iter_mut().map(|player| {
        let player_id = player.id;
        let world_ref = &world;
        async move {
            let player_world = world_ref.player_world(player_id);
            player.runner.run(player_world).await.map_or_else(
                |err| {
                    log::error!("Player {:?}: {}", player_id, err);
                    None
                },
                move |player_actions| {
                    Some(
                        player_actions
                            .into_iter()
                            .map(|action| validate_action(action, player_id, world_ref))
                            .filter_map(|action| match action {
                                Ok(action) => Some(action),
                                Err(err) => {
                                    log::error!("Player {:?}: invalid action: {}", player_id, err);
                                    None
                                }
                            })
                            .collect::<Vec<Action>>(),
                    )
                },
            )
        }
    }))
    .await
    .into_iter()
    .filter_map(|a| a)
    .flatten();

    // Run all actions on the world
    world.apply(actions.into_iter())
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
