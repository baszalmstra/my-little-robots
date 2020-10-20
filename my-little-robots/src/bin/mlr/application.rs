use bracket_lib::prelude::*;
use mlr::bracket_lib::draw_world;
use mlr::World;

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

pub fn run(world_receiver: async_watch::Receiver<World>) -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("My Little Robots")
        .build()?;
    let application_state = ApplicationState { world_receiver };

    // Run the main loop
    main_loop(context, application_state)
}
