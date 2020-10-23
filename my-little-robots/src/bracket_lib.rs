use crate::Map;
use crate::World;
use bracket_lib::prelude::*;
use mlr_api::{Coord, PlayerId, TileType, Unit, UnitId};
use std::collections::{HashMap, HashSet};

/// Returns the correct glyph for the TileType
pub fn glyph_for(coord: Coord, map: &Map) -> (impl Into<RGBA>, FontCharType) {
    let tile_type = map[coord];
    match tile_type {
        TileType::Wall => (WHITE, wall_glyph(map, coord.x, coord.y)),
        TileType::Floor => (GRAY, to_cp437('.')),
        TileType::Exit => (CYAN, to_cp437('>')),
    }
}

pub fn player_color(player: PlayerId) -> impl Into<RGBA> {
    match player.0 {
        0 => LIGHTGREEN,
        1 => BLUE_VIOLET,
        2 => ORANGERED,
        3 => GOLD,
        _ => GRAY,
    }
}

fn player_symbol(player: PlayerId) -> char {
    match player.0 {
        0 => '♦',
        1 => '♣',
        2 => '¶',
        3 => '♣',
        _ => '♥',
    }
}

pub fn player_glyph(player: PlayerId) -> FontCharType {
    to_cp437(player_symbol(player))
}

pub fn unit_glyph(unit: &Unit) -> FontCharType {
    player_glyph(unit.player)
}

pub fn is_revealed_and_wall(map: &Map, x: isize, y: isize) -> bool {
    x < 0
        || y < 0
        || x >= map.width as isize
        || y >= map.height as isize
        || map[(x, y)] == TileType::Wall
}

pub fn wall_glyph(map: &Map, x: isize, y: isize) -> FontCharType {
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

/// Draws the specified map
pub fn draw_map<F: Fn(Coord) -> f32>(map: &Map, is_visible: F, ctx: &mut BTerm) {
    let height = map.height as isize;
    let width = map.width as isize;

    for y in 0..height {
        for x in 0..width {
            let pos: Coord = (x, y).into();

            let (color, glyph) = glyph_for((x, y).into(), map);
            let mut color = color.into();
            color.a = 0.1 + (is_visible(pos) * 0.9);
            ctx.set(x, y, color, BLACK, glyph);
        }
    }
}

/// Draw the UI
pub fn draw_ui(world: &World, _units: &HashMap<UnitId, Coord>, ctx: &mut BTerm) {
    let map = &world.map;
    let mut ui_string = format!("Turn {}", world.turn);

    // TODO: change this to not happen each frame
    // Get unique players and sort them
    let mut players = HashSet::new();
    world.units.iter().for_each(|u| {
        players.insert(u.player);
    });
    let mut player_vector = Vec::with_capacity(players.len());
    for player in players.iter() {
        player_vector.push(player);
    }
    player_vector.sort_by(|a, b| a.0.cmp(&b.0));

    ui_string += &player_vector.iter().fold(String::new(), |acc, p| {
        acc + &format!(" Player {}: {}", p.0, player_symbol(**p))
    });
    ctx.print_centered(map.height - 1, ui_string);
}
