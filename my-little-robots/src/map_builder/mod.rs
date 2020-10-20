use super::Map;

mod cellular_automata;
mod prim;
mod snapshot;

use mlr_api::{Coord, Direction, TileType};
use rand::Rng;
pub use snapshot::{MapWithSnapshots, SnapshotableMap};

pub use cellular_automata::CellularAutomata;
pub use prim::PrimMazeBuilder;

pub fn new_map<B: MapBuilder>(width: usize, height: usize, builder: &mut B) -> Map {
    let mut map = Map::new_closed(width, height);
    builder.build(&mut map);
    map
}

pub fn new_map_with_history<B: MapBuilder>(
    width: usize,
    height: usize,
    builder: &mut B,
) -> Vec<Map> {
    let mut map: MapWithSnapshots = Map::new_closed(width, height).into();
    builder.build(&mut map);
    map.into()
}

pub trait MapBuilder {
    /// Constructs a map
    fn build<T: SnapshotableMap>(&mut self, map: &mut T);
}

pub struct SimpleMapBuilder;
impl MapBuilder for SimpleMapBuilder {
    fn build<T: SnapshotableMap>(&mut self, map: &mut T) {
        let mut rng = rand::thread_rng();

        // Carve out a huge open room
        map.with_snapshot(|map| {
            for y in 1..map.height - 1 {
                for x in 1..map.width - 1 {
                    map[(x, y)] = TileType::Floor;
                }
            }
        });

        // Spawn 400 random obstacles
        map.with_snapshot(|map| {
            for _i in 0..400 {
                let x = rng.gen_range(1, map.width - 2);
                let y = rng.gen_range(1, map.height - 2);
                map[(x, y)] = TileType::Wall;
            }
        });

        // Create an exit in one of the outer walls
        map.with_snapshot(|map| {
            let exit_direction = Direction::random(&mut rng);
            let exit_size = 10;
            let (mut start, dir): (Coord, Direction) = match exit_direction {
                Direction::Left => (
                    (0, rng.gen_range(0, map.height - exit_size)).into(),
                    Direction::Down,
                ),
                Direction::Right => (
                    (map.width - 1, rng.gen_range(0, map.height - exit_size)).into(),
                    Direction::Down,
                ),
                Direction::Up => (
                    (rng.gen_range(0, map.width - exit_size), 0).into(),
                    Direction::Right,
                ),
                Direction::Down => (
                    (rng.gen_range(0, map.width - exit_size), map.height - 1).into(),
                    Direction::Right,
                ),
            };
            for _i in 0..exit_size {
                map[start] = TileType::Exit;
                start += dir;
            }
        })
    }
}
