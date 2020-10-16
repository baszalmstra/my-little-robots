use crate::map::TileType;
use crate::Coord;
use crate::World;
use bracket_lib::prelude::*;
use std::sync::mpsc::Receiver;

struct ApplicationState {
    world: World,
    world_recv: Receiver<World>,
}

impl GameState for ApplicationState {
    fn tick(&mut self, ctx: &mut BTerm) {
        // Try to receive a new world
        if let Ok(world) = self.world_recv.try_recv() {
            self.world = world;
        }
        // Draw the world
        draw_world(&self.world, ctx);
    }
}

/// Returns the correct glyph for the TileType
fn glyph_for(tile_type: TileType) -> FontCharType {
    match tile_type {
        TileType::Wall => to_cp437('#'),
        TileType::Floor => to_cp437('.'),
    }
}

/// Draw the actual world
pub fn draw_world(world: &World, ctx: &mut BTerm) {
    let height = world.map.height as isize;
    let width = world.map.width as isize;

    // Draw map
    for y in 0..height {
        for x in 0..width {
            let glyph = glyph_for(world.map.tile_at((x, y)));
            ctx.set(x, y, GRAY, BLACK, glyph);
        }
    }

    // Draw units
    world.units.iter().for_each(|unit| {
        ctx.set(
            unit.location.x,
            unit.location.y,
            LIGHTGREEN,
            BLACK,
            to_cp437('R'),
        )
    })
}

pub fn run(world: World, world_recv: Receiver<World>) -> BError {
    let context = BTermBuilder::simple80x50().build()?;
    let application_state = ApplicationState { world, world_recv };

    // Run the main loop
    main_loop(context, application_state)
}
