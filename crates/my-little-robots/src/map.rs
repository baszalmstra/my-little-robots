use super::Coord;
use bracket_lib::prelude::{field_of_view_set, Algorithm2D, BaseMap, Point};
use mlr_api::{Direction, TileType};
use rand::prelude::IteratorRandom;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<TileType>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
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

    pub fn new_closed(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Wall; width * height],
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

    /// Returns all the coordinates that can be seen from the given location and within the given range
    pub fn field_of_view(&self, position: Coord, range: isize) -> HashSet<Coord> {
        field_of_view_set(Point::new(position.x, position.y), range as i32, self)
            .into_iter()
            .map(|p| Coord::new(p.x, p.y))
            .collect()
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

fn get_frontier_tiles(map: &Map, position: Coord) -> Vec<Coord> {
    let directions = Direction::all_directions();
    directions
        .into_iter()
        .filter_map(move |direction| {
            let mutation = Coord::from(direction);
            // Frontier tiles are set with a space of 2 tiles
            // and are blocked within the grid
            let new_coord = Coord::new(position.x + mutation.x * 2, position.y + mutation.y * 2);
            if map.in_bounds(new_coord) && map[new_coord] == TileType::Wall {
                Some(new_coord)
            } else {
                None
            }
        })
        .collect()
}

fn get_neighbor_tiles(map: &Map, position: Coord) -> Vec<Direction> {
    let directions = Direction::all_directions();
    directions
        .into_iter()
        .filter_map(move |direction| {
            let mutation = Coord::from(direction);
            // Frontier tiles are set with a space of 2 tiles
            // and are blocked within the grid
            let new_coord = Coord::new(position.x + mutation.x * 2, position.y + mutation.y * 2);
            if map.in_bounds(new_coord) && map[new_coord] == TileType::Floor {
                Some(direction)
            } else {
                None
            }
        })
        .collect()
}

/// A Grid consists of a 2 dimensional array of cells.
/// A Cell has 2 states: Blocked or Passage.
/// Start with a Grid full of Cells in state Blocked.
/// Pick a random Cell, set it to state Passage and Compute its frontier cells. A frontier cell of a Cell is a cell with distance 2 in state Blocked and within the grid.
/// While the list of frontier cells is not empty:
///     Pick a random frontier cell from the list of frontier cells.
///     Let neighbors(frontierCell) = All cells in distance 2 in state Passage. Pick a random neighbor and connect the frontier cell with the neighbor by setting the cell in-between to state Passage. Compute the frontier cells of the chosen frontier cell and add them to the frontier list. Remove the chosen frontier cell from the list of frontier cells.
pub(crate) fn new_map_prim(width: usize, height: usize) -> Map {
    let mut map = Map::new_closed(width, height);
    let mut rng = rand::thread_rng();

    let start = Coord::new(width as isize / 2, height as isize / 2);

    let mut visited = HashSet::new();
    visited.insert(start);
    map[start] = TileType::Floor;

    // Get walls around the start position
    let mut frontier_cells = get_frontier_tiles(&map, start);

    while !frontier_cells.is_empty() {
        // Select random frontier cell
        let index = rng.gen_range(0, frontier_cells.len());
        let frontier_cell = frontier_cells.remove(index);
        map[frontier_cell] = TileType::Floor;

        // Select neighbors
        let neighbors = get_neighbor_tiles(&map, frontier_cell);
        let between_dir = neighbors[rng.gen_range(0, neighbors.len())];

        // Create passage in between
        let mutation = Coord::from(between_dir);
        let in_between = Coord::new(frontier_cell.x + mutation.x, frontier_cell.y + mutation.y);
        map[in_between] = TileType::Floor;

        // Append new walls
        let new_frontier = get_frontier_tiles(&map, frontier_cell);
        for new_frontier_cell in new_frontier {
            if !visited.contains(&new_frontier_cell) {
                frontier_cells.push(new_frontier_cell);
                visited.insert(new_frontier_cell);
            }
        }
    }

    // Set a random exit for now
    if let Some((tile_idx, _)) = map
        .tiles
        .iter()
        .enumerate()
        .filter(|t| *t.1 == TileType::Floor)
        .choose(&mut rng)
    {
        map.tiles[tile_idx] = TileType::Exit;
    }

    map
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
            Direction::Right,
        ),
        Direction::Down => (
            (rng.gen_range(0, width - exit_size), height - 1).into(),
            Direction::Right,
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
