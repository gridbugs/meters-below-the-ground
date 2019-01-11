extern crate meters_prototty;
extern crate prototty_glutin;
extern crate rand;

use meters_prototty::*;
use prototty_glutin::*;
use rand::Rng;
use std::time::Instant;

const USER_DIR: &'static str = "user";

fn main() {
    let size = Size::new(960, 720);
    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_title("Meters Below the Ground")
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_font_scale(16.0, 16.0)
        .with_cell_dimensions(Size::new(16, 16))
        .with_underline_position(14)
        .with_underline_width(1)
        .with_max_grid_size(Size::new(60, 45))
        .build()
        .unwrap();

    let storage = FileStorage::next_to_exe(USER_DIR, true).expect("Failed to find user dir");

    let seed = rand::thread_rng().gen();

    let mut app = App::new(Frontend::Glutin, storage, seed);

    let mut input_buffer = Vec::with_capacity(64);

    let mut last_instant = Instant::now();

    let mut running = true;

    let mut view = AppView::new(context.size());

    loop {
        view.set_size(context.size());
        context.render(&mut view, &app).unwrap();

        if !running {
            break;
        }

        let now = Instant::now();

        let duration = now - last_instant;
        last_instant = now;

        context.poll_input(|input| {
            input_buffer.push(input);
        });

        if let Some(control_flow) = app.tick(input_buffer.drain(..), duration, &view) {
            match control_flow {
                ControlFlow::Quit => running = false,
            }
        }
    }
}
