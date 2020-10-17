pub mod application;
pub mod map;
mod unit;

use async_trait::async_trait;
use futures::future::join_all;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use self::map::Map;
pub use self::unit::{Unit, UnitId};
use crate::map::{new_map_test, TileType};
use std::ops::{Add, AddAssign};

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

// Conversion from a tuple
impl From<(isize, isize)> for Coord {
    fn from(tup: (isize, isize)) -> Self {
        Coord::new(tup.0, tup.1)
    }
}

impl From<(usize, usize)> for Coord {
    fn from(v: (usize, usize)) -> Self {
        Coord::new(v.0 as isize, v.1 as isize)
    }
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
            Direction::Up => Coord::new(self.x, self.y - 1),
            Direction::Down => Coord::new(self.x, self.y + 1),
        }
    }
}

impl AddAssign<Direction> for Coord {
    fn add_assign(&mut self, rhs: Direction) {
        *self = *self + rhs;
    }
}

/// A direction
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// Returns a random direction
    pub fn random<Rng: rand::Rng>(r: &mut Rng) -> Self {
        match r.gen_range(0, 4) {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            _ => Direction::Down,
        }
    }
}

impl From<Direction> for Coord {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Left => Coord::new(-1, 0),
            Direction::Right => Coord::new(1, 0),
            Direction::Up => Coord::new(0, -1),
            Direction::Down => Coord::new(0, 1),
        }
    }
}

impl Default for World {
    fn default() -> World {
        World {
            map: new_map_test(80, 50),
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
        PlayerWorld {
            units: self
                .units
                .iter()
                .filter(|unit| unit.player == player_id)
                .cloned()
                .collect(),
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
        let map_ref = &self.map;
        self.units
            .iter()
            .filter(move |unit| map_ref[unit.location] == TileType::Exit)
    }
}

/// A `PlayerWorld` represents only the visible parts of a world for a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerWorld {
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
    async fn run(&mut self, input: RunnerInput) -> Result<RunnerOutput, RunnerError>;
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
pub struct RunnerInput {
    pub player_id: PlayerId,
    pub world: PlayerWorld,
    pub memory: PlayerMemory,
}

/// The output of a `PlayerRunner`
pub struct RunnerOutput {
    pub actions: Vec<PlayerAction>,
    pub memory: PlayerMemory,
}

// Implement `PlayerRunner` for a functions
#[async_trait]
impl<F> PlayerRunner for F
where
    F: FnMut(RunnerInput) -> Result<RunnerOutput, RunnerError> + Send,
{
    async fn run(&mut self, input: RunnerInput) -> Result<RunnerOutput, RunnerError> {
        (self)(input)
    }
}

pub type PlayerMemory = serde_json::value::Value;

/// Represents everything of a specific player.
pub struct Player {
    /// The unique id of this player
    pub id: PlayerId,

    /// The function to generate actions from the current state of the world
    pub runner: Box<dyn PlayerRunner>,

    /// The current player memory
    pub memory: PlayerMemory,
}

/// Runs a single turn on the world
pub async fn turn(players: &mut [Player], world: World) -> World {
    // Get the actions from all the players
    let actions = join_all(players.iter_mut().map(|player| {
        let player_id = player.id;
        let world_ref = &world;
        async move {
            // Let the runner run
            let runner_output = player
                .runner
                .run(RunnerInput {
                    player_id,
                    world: world_ref.player_world(player_id),
                    memory: player.memory.clone(),
                })
                .await;

            // Check the output for errors
            let output = match runner_output {
                Err(err) => {
                    log::error!("Player {:?}: {}", player_id, err);
                    return None;
                }
                Ok(output) => output,
            };

            // Validate all the actions
            let mut actions = Vec::new();
            for player_action in output.actions {
                match validate_action(player_action, player_id, world_ref) {
                    Err(err) => {
                        log::error!("Player {:?}: invalid action: {}", player_id, err);
                    }
                    Ok(action) => actions.push(action),
                }
            }

            // Store the memory of the player
            player.memory = output.memory;

            Some(actions)
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
