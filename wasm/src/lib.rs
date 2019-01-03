extern crate meters_prototty;
extern crate prototty;
extern crate prototty_wasm;
extern crate rand;

use meters_prototty::*;
use prototty::Input as ProtottyInput;
use prototty::Renderer;
use prototty_wasm::*;
use std::time::Duration;

pub struct WebApp {
    app: App<WasmStorage>,
    context: Context,
    view: AppView,
}

impl WebApp {
    fn new(seed: usize, storage: WasmStorage) -> Self {
        let app = App::new(Frontend::Wasm, storage, seed);
        let context = Context::new();
        let view = AppView::new(context.size());

        Self { app, context, view }
    }
    fn tick<I>(&mut self, inputs: I, period: Duration)
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        self.view.set_size(self.context.size());
        if let Some(control_flow) = self.app.tick(inputs, period, &self.view) {
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
pub unsafe extern "C" fn alloc_app(
    seed: usize,
    storage_buf: *const u8,
    storage_len: usize,
) -> *mut WebApp {
    let storage = WasmStorage::from_ptr(storage_buf, storage_len);
    alloc::into_boxed_raw(WebApp::new(seed, storage))
}

#[no_mangle]
pub unsafe fn tick(app: *mut WebApp, inputs: *const u64, num_inputs: usize, period_millis: f64) {
    let period = Duration::from_millis(period_millis as u64);
    let input_iter = input::js_event_input_iter(inputs, num_inputs);
    (*app).tick(input_iter, period);
}
