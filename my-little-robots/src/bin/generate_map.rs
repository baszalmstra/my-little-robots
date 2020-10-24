use bracket_lib::prelude::*;
use mlr::bracket_lib::draw_map;
use mlr::map_builder::new_map_with_history;
use mlr::Map;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("ERROR: {}", err);
        std::process::exit(1)
    }
}

fn try_main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_fancy_console(80, 50, "terminal8x8.png".to_string())
        .with_title("My Little Robots - Map Generator")
        .build()?;

    //let mut builder = mlr::map_builder::SimpleMapBuilder;
    //let mut builder = mlr::map_builder::PrimMazeBuilder;
    let mut builder = mlr::map_builder::CellularAutomata;

    let map_history = new_map_with_history(80, 50, &mut builder);

    main_loop(
        context,
        ApplicationState {
            map_history,
            index: 0,
        },
    )
}

struct ApplicationState {
    map_history: Vec<Map>,
    index: usize,
}

/// Draw the overlay over the map, is currently used to show the distance to the exit
pub fn draw_overlay(map: &Map, ctx: &mut BTerm) {
    let height = map.height as isize;
    let width = map.width as isize;

    for y in 0..height {
        for x in 0..width {
            if let Some(_coord) = map.get_distance_to_exit((x, y)) {
                ctx.set_fancy(
                    PointF::new(x as f32, y as f32),
                    1,
                    Radians(0.0f32),
                    PointF::new(0.5f32, 0.5f32),
                    RGBA::from_u8(10, 255, 10, 200),
                    RGBA::from_u8(0, 0, 0, 0),
                    to_cp437('x'),
                );
            }
        }
    }
}

impl GameState for ApplicationState {
    fn tick(&mut self, ctx: &mut BTerm) {
        match ctx.key {
            Some(VirtualKeyCode::Space) | Some(VirtualKeyCode::Right)
                if self.index < self.map_history.len() - 1 =>
            {
                self.index += 1
            }
            Some(VirtualKeyCode::Left) if self.index > 0 => self.index -= 1,
            Some(VirtualKeyCode::End) => self.index = self.map_history.len() - 1,
            _ => {}
        };

        // Draw the world
        ctx.cls();
        ctx.set_active_console(0);
        draw_map(&self.map_history[self.index], |_| 1.0, ctx);
        ctx.set_active_console(1);
        draw_overlay(&self.map_history[self.index], ctx);
    }
}
