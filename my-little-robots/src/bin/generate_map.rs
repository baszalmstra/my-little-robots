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

impl GameState for ApplicationState {
    fn tick(&mut self, ctx: &mut BTerm) {
        match ctx.key {
            Some(VirtualKeyCode::Space) | Some(VirtualKeyCode::Right)
                if self.index < self.map_history.len() - 1 =>
            {
                self.index += 1
            }
            Some(VirtualKeyCode::Left) if self.index > 0 => self.index -= 1,
            _ => {}
        };

        // Draw the world
        ctx.cls();
        draw_map(&self.map_history[self.index], |_| 1.0, ctx);
    }
}
