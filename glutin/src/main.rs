extern crate prototty;
extern crate prototty_glutin;
extern crate punchcards_prototty;

use std::time::Instant;
use prototty::Renderer;
use prototty_glutin::*;
use punchcards_prototty::{App, AppView, ControlFlow};

fn main() {

    let mut context = ContextBuilder::new_with_font(include_bytes!("fonts/PxPlus_IBM_CGAthin.ttf"))
        .with_bold_font(include_bytes!("fonts/PxPlus_IBM_CGA.ttf"))
        .with_window_dimensions(640, 480)
        .with_font_scale(32.0, 32.0)
        .with_cell_dimensions(32, 32)
        .with_underline_position(28)
        .with_underline_width(2)
        .with_max_grid_size(20, 20)
        .build().unwrap();

    let mut app = App::new();
    let mut input_buffer = Vec::with_capacity(64);

    let mut last_instant = Instant::now();

    let mut running = true;

    loop {
        context.render(&AppView, &app).unwrap();

        if !running {
            break;
        }

        let now = Instant::now();

        let duration = now - last_instant;
        last_instant = now;

        context.poll_input(|input| {
            input_buffer.push(input);
        });

        if let Some(control_flow) = app.tick(input_buffer.drain(..), duration) {
            match control_flow {
                ControlFlow::Quit => running = false,
            }
        }

    }
}
