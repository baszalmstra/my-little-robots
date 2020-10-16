use super::Coord;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct Map {
    width: usize,
    height: usize,
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
            || position.x < self.width as isize
            || position.y >= 0
            || position.y < self.height as isize
    }

    /// Checks if this tile can be entered
    pub fn can_enter_tile(&self, position: Coord) -> bool {
        self.in_bounds(position)
            && self.tiles[self.map_coord_from_coord(position).0] == TileType::Floor
    }

    /// Computes the `MapCoord` from the specified `Coord`
    fn map_coord_from_coord(&self, position: Coord) -> MapCoord {
        MapCoord(position.x as usize + position.y as usize * self.width)
    }
}
