extern crate punchcards_prototty;
extern crate prototty;
extern crate prototty_unix;

use std::time::Duration;
use std::thread;
use punchcards_prototty::{App, AppView, ControlFlow};
use prototty_unix::Context;
use prototty::Renderer;

const TICK_MILLIS: u64 = 33;

fn main() {
    let mut app = App::new();
    let mut context = Context::new().unwrap();

    let view = AppView::new();

    loop {
        context.render(&view, &app).unwrap();
        thread::sleep(Duration::from_millis(TICK_MILLIS));

        if let Some(control_flow) = app.tick(context.drain_input().unwrap(), Duration::from_millis(TICK_MILLIS)) {
            match control_flow {
                ControlFlow::Quit => break,
            }
        }
    }
}
