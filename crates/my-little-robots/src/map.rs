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

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![TileType::Floor; width * height],
        }
    }
}
