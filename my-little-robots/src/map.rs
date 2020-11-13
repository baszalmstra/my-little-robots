use super::Coord;
use bracket_lib::prelude::{field_of_view_set, Algorithm2D, BaseMap, Point};
use mlr_api::TileType;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub(crate) tiles: Vec<TileType>,
    pub(crate) distance_to_exit: Vec<Option<usize>>,
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
            distance_to_exit: vec![None; width * height],
        }
    }

    pub fn new_closed(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Wall; width * height],
            distance_to_exit: vec![None; width * height],
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

    pub fn get_distance_to_exit<T: Into<Coord>>(&self, position: T) -> Option<usize> {
        let coord = position.into();
        let index = coord.x as usize + coord.y as usize * self.width;
        self.distance_to_exit[index]
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
