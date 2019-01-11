extern crate meters_prototty;
extern crate prototty_unix;
extern crate rand;

use meters_prototty::*;
use prototty_unix::{Context, FileStorage};
use rand::Rng;
use std::thread;
use std::time::Duration;

const USER_DIR: &'static str = "user";
const TICK_MILLIS: u64 = 33;

fn main() {
    let storage = FileStorage::next_to_exe(USER_DIR, true).expect("Failed to find user dir");

    let mut context = Context::new().unwrap();

    let mut app = App::new(Frontend::Unix, storage, rand::thread_rng().gen());

    let mut view = AppView::new(context.size().unwrap());

    loop {
        view.set_size(context.size().unwrap());

        context.render(&mut view, &app).unwrap();
        thread::sleep(Duration::from_millis(TICK_MILLIS));

        if let Some(control_flow) = app.tick(
            context.drain_input().unwrap(),
            Duration::from_millis(TICK_MILLIS),
            &view,
        ) {
            match control_flow {
                ControlFlow::Quit => break,
            }
        }
    }
}
