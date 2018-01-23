extern crate rand;
extern crate prototty;
extern crate prototty_wasm;
extern crate punchcards_prototty;

use std::time::Duration;
use rand::{StdRng, SeedableRng};
use prototty_wasm::*;
use prototty::Renderer;
use prototty::Input as ProtottyInput;

use punchcards_prototty::{App, AppView, ControlFlow};

pub struct WebApp {
    app: App<StdRng>,
    context: Context,
    view: AppView,
}

impl WebApp {
    fn new(seed: usize) -> Self {
        let rng = StdRng::from_seed(&[seed]);
        let app = App::new(rng);
        let context = Context::new();
        let view = AppView::new();

        Self {
            app,
            context,
            view,
        }
    }
    fn tick<I>(&mut self, inputs: I, period: Duration)
        where I: IntoIterator<Item=ProtottyInput>,
    {
        if let Some(control_flow) = self.app.tick(inputs, period) {
            match control_flow {
                ControlFlow::Quit => {
                    self.context.quit();
                    return;
                }
            }
        }
        self.context.render(&mut self.view, &self.app).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn alloc_app(seed: usize) -> *mut WebApp {
    alloc::into_boxed_raw(WebApp::new(seed))
}

#[no_mangle]
pub unsafe fn tick(app: *mut WebApp,
                   key_codes: *const u8,
                   key_mods: *const u8,
                   num_inputs: usize,
                   period_millis: f64) {

    let period = Duration::from_millis(period_millis as u64);

    let input_iter = input::js_event_input_iter(key_codes, key_mods, num_inputs);
    (*app).tick(input_iter, period);
}
