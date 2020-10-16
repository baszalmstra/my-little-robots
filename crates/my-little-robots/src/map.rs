use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<TileType>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Floor; width * height],
        }
    }

    /// Transform 2d coordinate to 1D idx
    pub fn map_idx(&self, x: usize, y: usize) -> usize {
        self.width * y + x
    }

    /// Get the TileType at x and y coordinate
    pub fn tile_at(&self, x: usize, y: usize) -> TileType {
        self.tiles[self.map_idx(x, y)]
    }
}
