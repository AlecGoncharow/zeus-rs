use my_engine::context::Context;
use my_engine::event::EventHandler;

struct State {
    frame: u32,
}

impl EventHandler for State {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), ()> {
        self.frame += 1;
        println!("Drawing frame: {}", self.frame);
        ctx.render();
        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> Result<(), ()> {
        println!("Updating");
        Ok(())
    }
}

fn main() {
    let (ctx, event_loop) = Context::new();
    let my_game = State { frame: 0 };

    let _ = my_engine::event::run(event_loop, ctx, my_game);
}
