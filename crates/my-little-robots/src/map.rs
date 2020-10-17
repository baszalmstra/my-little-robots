use super::Coord;
use crate::Direction;
use bracket_lib::prelude::{field_of_view_set, Algorithm2D, BaseMap, Point};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
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
        field_of_view_set(position.into(), range as i32, self)
            .into_iter()
            .map(Into::into)
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

/// Get the type of the  tiles around the given coordinate
/// in the directions RIGHT, LEFT, UP, DOWN
/// returns a tuple with the coordinate and the TileType
fn get_frontier_tiles(
    map: &Map,
    position: Coord,
) -> impl Iterator<Item = (Coord, Direction, &TileType)> {
    let directions = Direction::all_directions();
    directions.into_iter().filter_map(move |direction| {
        let mutation = Coord::from(direction);
        // Frontier tiles are set with a space of 2 tiles
        // and are blocked within the grid
        let new_coord = Coord::new(position.x + mutation.x * 2, position.y + mutation.y * 2);
        if map.in_bounds(new_coord) && map[new_coord] == TileType::Wall {
            Some((new_coord, direction, &map[new_coord]))
        } else {
            None
        }
    })
}

fn get_neighbor_tiles(map: &Map, position: Coord) -> Vec<(Coord, Direction)> {
    let directions = Direction::all_directions();
    directions
        .into_iter()
        .filter_map(move |direction| {
            let mutation = Coord::from(direction);
            // Frontier tiles are set with a space of 2 tiles
            // and are blocked within the grid
            let new_coord = Coord::new(position.x + mutation.x * 2, position.y + mutation.y * 2);
            if map.in_bounds(new_coord) && map[new_coord] == TileType::Floor {
                Some((new_coord, direction))
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn new_map_prim(width: usize, height: usize) -> Map {
    let mut map = Map::new_closed(width, height);
    let mut rng = rand::thread_rng();

    let start = Coord::new(width as isize / 2, height as isize / 2);

    let mut visited = HashSet::new();
    visited.insert(start);
    map[start] = TileType::Floor;

    // Get walls around the start position
    let mut frontier_cells = get_frontier_tiles(&map, start)
        .map(|t| (t.0, t.1))
        .collect::<Vec<(Coord, Direction)>>();

    dbg!(start);
    while !frontier_cells.is_empty() {
        // Select random frontier cell
        let index = rng.gen_range(0, frontier_cells.len());
        let (frontier_cell, _) = frontier_cells.remove(index);
        map[frontier_cell] = TileType::Floor;
        dbg!(frontier_cell);
        let neighbors = get_neighbor_tiles(&map, frontier_cell);
        dbg!(&neighbors);
        let (_, between_dir) = neighbors[rng.gen_range(0, neighbors.len())];
        // Create passage in between
        let mutation = Coord::from(between_dir);
        let in_between = Coord::new(frontier_cell.x + mutation.x, frontier_cell.y + mutation.y);
        map[in_between] = TileType::Floor;

        //// Append new walls
        let new_frontier = get_frontier_tiles(&map, frontier_cell).map(|t| (t.0, t.1));
        for new_frontier_cell in new_frontier {
            if !visited.contains(&new_frontier_cell.0) {
                frontier_cells.push(new_frontier_cell);
                visited.insert(new_frontier_cell.0);
            }
        }

        //if !visited.contains(&to) {
        //// Set the visited as floor
        //map[to] = TileType::Floor;
        //// Set the tile in-between as a floor
        //let mutation = Coord::from(dir);
        //let in_between = Coord::new(from.x + mutation.x, from.y + mutation.y);
        //if map.in_bounds(in_between) {
        //map[in_between] = TileType::Floor;
        //}
        //// We have visited this
        //visited.insert(to);
        //// Append new walls
        //let mut new_frontier = get_frontier_tiles(&map, to).map(|t| (t.0, t.1)).collect();
        //frontier_cells.append(&mut new_frontier);

        //// Set new from
        //from = to;
        //}
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
