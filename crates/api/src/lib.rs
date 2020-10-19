use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt::Debug;
use std::time::Duration;
use thiserror::Error;

/// A `PlayerId` uniquely describes a single Player
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PlayerId(pub usize);

impl std::fmt::Debug for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A coordinate in the world
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(from = "(isize, isize)", into = "(isize, isize)")]
pub struct Coord {
    pub x: isize,
    pub y: isize,
}

impl Coord {
    /// Constructs a new `Coord` from its components
    pub fn new<T: TryInto<isize>>(x: T, y: T) -> Self {
        Coord {
            x: x.try_into().ok().unwrap_or(0),
            y: y.try_into().ok().unwrap_or(0),
        }
    }
}

// Conversion from a tuple and back
impl<T: TryInto<isize>> From<(T, T)> for Coord {
    fn from(tup: (T, T)) -> Self {
        Coord::new(tup.0, tup.1)
    }
}

impl<T: From<isize>> From<Coord> for (T, T) {
    fn from(coord: Coord) -> Self {
        (coord.x.into(), coord.y.into())
    }
}

/// Unique identifier of a specific `Unit`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UnitId(pub usize);

/// A `Unit` describes a single unit
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub player: PlayerId,
    pub location: Coord,
}

/// A `PlayerWorld` represents only the visible parts of a world for a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerWorld {
    pub units: Vec<Unit>,
    pub tiles: Vec<PlayerTile>,
}

/// The type for a single tile in the world
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    Wall,
    Floor,
    Exit,
}

impl TileType {
    /// Returns true if this is a type of tile that can be entered
    pub fn can_enter(self) -> bool {
        matches!(self, TileType::Floor | TileType::Exit)
    }
}

/// Represents a tile visible to a specific player
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerTile {
    pub coord: Coord,
    #[serde(rename = "type")]
    pub tile_type: TileType,
}

/// Describes a possible action that can be performed in the world as ordered by a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PlayerAction {
    Move { unit: UnitId, direction: Direction },
}

/// A direction
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
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

impl std::ops::Add<Direction> for Coord {
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

impl std::ops::AddAssign<Direction> for Coord {
    fn add_assign(&mut self, rhs: Direction) {
        *self = *self + rhs;
    }
}

impl Direction {
    /// Returns a random direction
    pub fn random<Rng: rand::Rng>(rng: &mut Rng) -> Self {
        match rng.gen_range(0, 4) {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            _ => Direction::Down,
        }
    }

    /// Returns all directions
    pub fn all_directions() -> Vec<Direction> {
        vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }
}

pub type PlayerMemory = serde_json::value::Value;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum RunnerError {
    #[error("internal error")]
    InternalError,

    #[error("the program errored while initializing")]
    InitError(String),

    #[error("the program exited before it returned any data")]
    NoData,

    #[error("IO error: {0}")]
    IO(String),

    #[error("the program took too long, past the time limit of {0:?}")]
    Timeout(Duration),

    #[error("Program returned invalid data")]
    DataError(String),
}

impl From<serde_json::Error> for RunnerError {
    fn from(err: serde_json::Error) -> Self {
        Self::DataError(err.to_string())
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

/// The input for a `PlayerRunner`
#[derive(Serialize, Deserialize)]
pub struct PlayerInput<T: Debug = PlayerMemory> {
    pub player_id: PlayerId,
    pub turn: usize,
    pub world: PlayerWorld,
    pub memory: T,
}

/// The output of a `PlayerRunner`
#[derive(Serialize, Deserialize)]
pub struct PlayerOutput<T: Debug = PlayerMemory> {
    pub actions: Vec<PlayerAction>,
    pub memory: T,
}
