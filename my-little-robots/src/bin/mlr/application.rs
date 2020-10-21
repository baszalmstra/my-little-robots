use bracket_lib::prelude::*;
use mlr::bracket_lib::{draw_map, player_color, unit_glyph};
use mlr::World;
use mlr_api::{Coord, UnitId};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

#[derive(Clone)]
struct AnimatedWorld {
    world: World,
    unit_locations: HashMap<UnitId, Coord>,
    visible_tiles: HashSet<Coord>,
}

impl From<World> for AnimatedWorld {
    fn from(world: World) -> Self {
        let unit_locations = world
            .units
            .iter()
            .map(|unit| (unit.id, unit.location))
            .collect();

        let visible_tiles = world
            .units
            .iter()
            .map(|unit| world.map.field_of_view(unit.location, 7))
            .flatten()
            .collect();

        AnimatedWorld {
            world,
            unit_locations,
            visible_tiles,
        }
    }
}

struct ApplicationState {
    world_receiver: async_watch::Receiver<World>,
    last_world: AnimatedWorld,
    world: AnimatedWorld,
    animation_time: f32,
}

impl ApplicationState {
    fn do_world_turn(&mut self) {
        let world = self.world_receiver.borrow();

        if world.turn != self.world.world.turn {
            self.animation_time = 0.0;

            std::mem::swap(&mut self.world, &mut self.last_world);
            self.world = world.clone().into();
        }
    }
}

impl GameState for ApplicationState {
    fn tick(&mut self, ctx: &mut BTerm) {
        // Try to receive a new world
        self.do_world_turn();

        // Clear the screen
        ctx.cls();

        // Draw the world
        let is_visible = |coord: Coord| {
            let was_contained = if self.last_world.visible_tiles.contains(&coord) {
                1.0
            } else {
                0.0
            };
            let currently_contained = if self.world.visible_tiles.contains(&coord) {
                1.0
            } else {
                0.0
            };
            was_contained + (currently_contained - was_contained) * self.animation_time
        };

        // Draw map
        ctx.set_active_console(0);
        draw_map(&self.world.world.map, is_visible, ctx);

        // Draw units
        ctx.set_active_console(1);
        for unit in self.world.world.units.iter() {
            let current_position =
                PointF::new(unit.location.x as f32 - 0.0, unit.location.y as f32 + 1.0);
            let position =
                if let Some(previous_location) = self.last_world.unit_locations.get(&unit.id) {
                    let previous_position = PointF::new(
                        previous_location.x as f32 - 0.0,
                        previous_location.y as f32 + 1.0,
                    );
                    previous_position + (current_position - previous_position) * self.animation_time
                } else {
                    current_position
                };

            ctx.set_fancy(
                position,
                1,
                Radians(0.0),
                (1.0, 1.0).into(),
                player_color(unit.player),
                BLACK,
                unit_glyph(unit),
            )
        }

        let frame_animation_time = 100.0;
        self.animation_time =
            (self.animation_time + ctx.frame_time_ms / frame_animation_time).min(1.0);
    }
}

pub fn run(world_receiver: async_watch::Receiver<World>) -> BError {
    let context = BTermBuilder::simple80x50()
        .with_fancy_console(80, 50, "terminal8x8.png".to_string())
        .with_title("My Little Robots")
        .build()?;
    let world: AnimatedWorld = world_receiver.borrow().deref().clone().into();
    let application_state = ApplicationState {
        world_receiver,
        last_world: world.clone(),
        world,
        animation_time: 1.0,
    };

    // Run the main loop
    main_loop(context, application_state)
}
