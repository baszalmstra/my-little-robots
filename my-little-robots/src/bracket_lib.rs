use crate::{Map, World};
use bracket_lib::prelude::*;
use mlr_api::{Coord, PlayerId, TileType, Unit};
use std::collections::HashSet;

/// Returns the correct glyph for the TileType
fn glyph_for(coord: Coord, map: &Map) -> (impl Into<RGBA>, FontCharType) {
    let tile_type = map[coord];
    match tile_type {
        TileType::Wall => (WHITE, wall_glyph(map, coord.x, coord.y)),
        TileType::Floor => (GRAY, to_cp437('.')),
        TileType::Exit => (CYAN, to_cp437('>')),
    }
}

fn player_color(player: PlayerId) -> impl Into<RGBA> {
    match player.0 {
        0 => LIGHTGREEN,
        1 => BLUE_VIOLET,
        2 => ORANGERED,
        3 => GOLD,
        _ => GRAY,
    }
}

fn unit_glyph(unit: &Unit) -> FontCharType {
    match unit.player.0 {
        0 => to_cp437('♦'),
        1 => to_cp437('♣'),
        2 => to_cp437('¶'),
        3 => to_cp437('♣'),
        _ => to_cp437('♥'),
    }
}

fn is_revealed_and_wall(map: &Map, x: isize, y: isize) -> bool {
    x < 0
        || y < 0
        || x >= map.width as isize
        || y >= map.height as isize
        || map[(x, y)] == TileType::Wall
}

fn wall_glyph(map: &Map, x: isize, y: isize) -> FontCharType {
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 10,   // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}

/// Draw the actual world
pub fn draw_world(world: &World, ctx: &mut BTerm) {
    let visible_tiles: HashSet<Coord> = world
        .units
        .iter()
        .map(|unit| world.map.field_of_view(unit.location, 7))
        .flatten()
        .collect();

    let is_visible = |coord: Coord| visible_tiles.contains(&coord);

    // Draw map
    draw_map(&world.map, is_visible, ctx);

    // Draw units
    world.units.iter().for_each(|unit| {
        ctx.set(
            unit.location.x,
            unit.location.y,
            player_color(unit.player),
            BLACK,
            unit_glyph(unit),
        )
    })
}

/// Draws the specified map
pub fn draw_map<F: Fn(Coord) -> bool>(map: &Map, is_visible: F, ctx: &mut BTerm) {
    let height = map.height as isize;
    let width = map.width as isize;

    for y in 0..height {
        for x in 0..width {
            let pos: Coord = (x, y).into();

            let (color, glyph) = glyph_for((x, y).into(), map);
            let color = if !is_visible(pos) {
                let mut color = color.into();
                color.a = 0.2;
                color
            } else {
                color.into()
            };
            ctx.set(x, y, color, BLACK, glyph);
        }
    }
}
