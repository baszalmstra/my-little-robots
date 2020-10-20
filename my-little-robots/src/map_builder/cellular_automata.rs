use super::{MapBuilder, SnapshotableMap, TileType};
use rand::Rng;

pub struct CellularAutomata;
impl MapBuilder for CellularAutomata {
    fn build<T: SnapshotableMap>(&mut self, map: &mut T) {
        let mut rng = rand::thread_rng();

        // First we completely randomize the map, setting 55% of it to be floor.
        map.with_snapshot(|map| {
            for y in 1..map.height - 1 {
                for x in 1..map.width - 1 {
                    let coord = (x, y);
                    let roll = rng.gen_range(0, 100);
                    if roll > 55 {
                        map[coord] = TileType::Floor
                    } else {
                        map[coord] = TileType::Wall
                    }
                }
            }
        });

        // Now we iteratively apply cellular automata rules
        for _i in 0..15 {
            map.with_snapshot(|map| {
                let mut newtiles = map.clone();

                for y in 1..map.height - 1 {
                    for x in 1..map.width - 1 {
                        let mut neighbors = 0;
                        if map[(x - 1, y)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x + 1, y)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x, y - 1)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x, y + 1)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x + 1, y - 1)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x - 1, y - 1)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x - 1, y + 1)] == TileType::Wall {
                            neighbors += 1;
                        }
                        if map[(x + 1, y + 1)] == TileType::Wall {
                            neighbors += 1;
                        }

                        if neighbors > 4 || neighbors == 0 {
                            newtiles[(x, y)] = TileType::Wall;
                        } else {
                            newtiles[(x, y)] = TileType::Floor;
                        }
                    }
                }

                *map = newtiles;
            });
        }
    }
}
