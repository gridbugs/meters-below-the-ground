extern crate meters_prototty;
extern crate prototty_wasm_input;
extern crate prototty_wasm_render;
extern crate prototty_wasm_storage;
extern crate wasm_bindgen;

use meters_prototty::*;
use prototty_wasm_render::*;
use prototty_wasm_storage::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

pub use prototty_wasm_input::InputBuffer;

#[wasm_bindgen]
pub struct WebApp {
    app_view: AppView,
    app: App<WasmStorage>,
    js_grid: JsGrid,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, js_grid: JsGrid, js_byte_storage: JsByteStorage) -> Self {
        let wasm_storage = WasmStorage::new(js_byte_storage);
        let app = App::new(Frontend::Wasm, wasm_storage, seed as usize);
        let app_view = AppView::new(Size::new(60, 45));
        Self {
            app_view,
            app,
            js_grid,
        }
    }

    pub fn tick(&mut self, input_buffer: &InputBuffer, period_ms: f64) {
        let period = Duration::from_millis(period_ms as u64);
        self.app.tick(input_buffer.iter(), period, &self.app_view);
        self.js_grid.render(&mut self.app_view, &self.app);
    }
}
