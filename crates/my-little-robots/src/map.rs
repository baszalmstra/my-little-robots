use super::Coord;
use crate::Direction;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Exit,
}

impl TileType {
    /// Returns true if this is a type of tile that can be entered
    fn can_enter(self) -> bool {
        matches!(self, TileType::Floor | TileType::Exit)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<TileType>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MapCoord(usize);

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Floor; width * height],
        }
    }

    /// Checks if the given coordinate is within the bounds of the map
    pub fn in_bounds(&self, position: Coord) -> bool {
        position.x >= 0
            && position.x < self.width as isize
            && position.y >= 0
            && position.y < self.height as isize
    }

    /// Checks if this tile can be entered
    pub fn can_enter_tile(&self, position: Coord) -> bool {
        self.in_bounds(position) && self[position].can_enter()
    }
}

impl<T: Into<Coord>> Index<T> for Map {
    type Output = TileType;

    fn index(&self, index: T) -> &Self::Output {
        let coord = index.into();
        let index = coord.x as usize + coord.y as usize * self.width;
        &self.tiles[index]
    }
}

impl<T: Into<Coord>> IndexMut<T> for Map {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let coord = index.into();
        let index = coord.x as usize + coord.y as usize * self.width;
        &mut self.tiles[index]
    }
}

/// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
/// look awful.
pub(crate) fn new_map_test(width: usize, height: usize) -> Map {
    let mut map = Map::new(width, height);

    // Make the boundary walls
    for x in 0..width {
        map[(x, 0)] = TileType::Wall;
        map[(x, height - 1)] = TileType::Wall;
    }

    for y in 0..height {
        map[(0, y)] = TileType::Wall;
        map[(width - 1, y)] = TileType::Wall;
    }

    // Sample a random direction for the exit
    let mut rng = rand::thread_rng();
    let exit_direction = Direction::random(&mut rng);
    let exit_size = 10;
    let (mut start, dir): (Coord, Direction) = match exit_direction {
        Direction::Left => (
            (0, rng.gen_range(0, height - exit_size)).into(),
            Direction::Down,
        ),
        Direction::Right => (
            (width - 1, rng.gen_range(0, height - exit_size)).into(),
            Direction::Down,
        ),
        Direction::Up => (
            (rng.gen_range(0, width - exit_size), 0).into(),
            Direction::Left,
        ),
        Direction::Down => (
            (rng.gen_range(0, width - exit_size), height - 1).into(),
            Direction::Left,
        ),
    };
    for _i in 0..exit_size {
        map[start] = TileType::Exit;
        start += dir;
    }

    // Spawn random obstacles
    for _i in 0..400 {
        let x = rng.gen_range(1, width - 2);
        let y = rng.gen_range(1, height - 2);
        map[(x, y)] = TileType::Wall;
    }

    map
}
