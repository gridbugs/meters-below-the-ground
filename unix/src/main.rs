extern crate prototty;
extern crate prototty_file_storage;
extern crate prototty_unix;
extern crate punchcards_prototty;
extern crate rand;

use std::time::Duration;
use std::thread;
use rand::Rng;
use prototty_unix::Context;
use prototty::Renderer;
use prototty_file_storage::FileStorage;
use punchcards_prototty::*;

const USER_DIR: &'static str = "user";
const TICK_MILLIS: u64 = 33;

fn main() {
    let storage = FileStorage::next_to_exe(USER_DIR, true).expect("Failed to find user dir");

    let mut context = Context::new().unwrap();

    let mut app = App::new(Frontend::Unix, storage, rand::thread_rng().gen());

    let mut view = AppView::new(context.size());

    loop {
        view.set_size(context.size());

        context.render(&mut view, &app).unwrap();
        thread::sleep(Duration::from_millis(TICK_MILLIS));

        if let Some(control_flow) = app.tick(
            context.drain_input().unwrap(),
            Duration::from_millis(TICK_MILLIS),
        ) {
            match control_flow {
                ControlFlow::Quit => break,
            }
        }
    }
}
