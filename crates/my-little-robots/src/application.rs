use crate::map::Map;
use crate::Unit;
use crate::World;
use bracket_lib::prelude::*;
use mlr_api::{Coord, PlayerId, TileType};
use std::collections::HashSet;

struct ApplicationState {
    world_receiver: async_watch::Receiver<World>,
}

impl GameState for ApplicationState {
    fn tick(&mut self, ctx: &mut BTerm) {
        // Try to receive a new world
        let world = self.world_receiver.borrow();

        // Draw the world
        ctx.cls();
        draw_world(&world, ctx);
    }
}

/// Returns the correct glyph for the TileType
fn glyph_for(coord: Coord, map: &Map) -> (impl Into<RGBA>, FontCharType) {
    let tile_type = map[coord];
    match tile_type {
        TileType::Wall => (WHITE, to_cp437('#')),
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

/// Draw the actual world
pub fn draw_world(world: &World, ctx: &mut BTerm) {
    let height = world.map.height as isize;
    let width = world.map.width as isize;

    let visible_tiles: HashSet<Coord> = world
        .units
        .iter()
        .map(|unit| world.map.field_of_view(unit.location, 7))
        .flatten()
        .collect();

    // Draw map
    for y in 0..height {
        for x in 0..width {
            let pos: Coord = (x, y).into();

            let (color, glyph) = glyph_for((x, y).into(), &world.map);
            let color = if !visible_tiles.contains(&pos) {
                let mut color = color.into();
                color.a = 0.2;
                color
            } else {
                color.into()
            };
            ctx.set(x, y, color, BLACK, glyph);
        }
    }

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

pub fn run(world_receiver: async_watch::Receiver<World>) -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("My Little Robots")
        .build()?;
    let application_state = ApplicationState { world_receiver };

    // Run the main loop
    main_loop(context, application_state)
}
