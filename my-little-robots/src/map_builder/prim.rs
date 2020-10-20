use super::{Coord, Direction, Map, MapBuilder, SnapshotableMap, TileType};
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::HashSet;

/// Calculate whether these cells can be selected as a frontier or neighbor bound
/// TODO: change this to be generic, because the cells skip 2 places it depends
/// on the starting position whether the border gets filled, for now we assume
/// a staring position in the center and then make the map smaller so that the left
/// and bottom border are not used
fn in_frontier_bounds(map: &Map, position: Coord) -> bool {
    position.x >= 1
        && position.x < (map.width) as isize
        && position.y >= 0
        && position.y < (map.height - 1) as isize
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
            if in_frontier_bounds(map, new_coord) && map[new_coord] == TileType::Wall {
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
        .filter(move |direction| {
            let mutation = Coord::from(*direction);
            // Neighbor tiles are set with a space of 2 tiles
            // and are exposed within the grid
            let new_coord = Coord::new(position.x + mutation.x * 2, position.y + mutation.y * 2);
            in_frontier_bounds(map, new_coord) && map[new_coord] == TileType::Floor
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
pub struct PrimMazeBuilder;
impl MapBuilder for PrimMazeBuilder {
    fn build<T: SnapshotableMap>(&mut self, map: &mut T) {
        let mut rng = rand::thread_rng();

        let mut visited = HashSet::new();

        // Add the start
        let mut frontier_cells = map.with_snapshot(|map| {
            let start = Coord::new(map.width as isize / 2, map.height as isize / 2);
            visited.insert(start);
            map[start] = TileType::Floor;
            get_frontier_tiles(&map, start)
        });

        while !frontier_cells.is_empty() {
            map.with_snapshot(|map| {
                // Select random frontier cell
                let index = rng.gen_range(0, frontier_cells.len());
                let frontier_cell = frontier_cells.remove(index);
                map[frontier_cell] = TileType::Floor;

                // Select neighbors
                let neighbors = get_neighbor_tiles(&map, frontier_cell);
                let between_dir = neighbors[rng.gen_range(0, neighbors.len())];

                // Create passage in between
                let in_between = frontier_cell + between_dir;
                map[in_between] = TileType::Floor;

                // Append new walls
                let new_frontier = get_frontier_tiles(&map, frontier_cell);
                for new_frontier_cell in new_frontier {
                    if !visited.contains(&new_frontier_cell) {
                        frontier_cells.push(new_frontier_cell);
                        visited.insert(new_frontier_cell);
                    }
                }
            });
        }

        // Test for closing of the sides, this was not very nice
        // but might be useful in the future
        // Close off all the sides
        //let map_width = map.width;
        //let map_height = map.height;
        //for x in 0..map_width {
        //let top = Coord::new(x, 0);
        //let bot = Coord::new(x, map_height - 1);
        //map[top] = TileType::Wall;
        //map[bot] = TileType::Wall;
        //}

        //for y in 0..map_height {
        //let left = Coord::new(0, y);
        //let right = Coord::new(map_width - 1, y);
        //map[left] = TileType::Wall;
        //map[right] = TileType::Wall;
        //}

        // Set a random exit for now
        map.with_snapshot(|map| {
            if let Some((tile_idx, _)) = map
                .tiles
                .iter()
                .enumerate()
                .filter(|t| *t.1 == TileType::Floor)
                .choose(&mut rng)
            {
                map.tiles[tile_idx] = TileType::Exit;
            }
        });
    }
}
