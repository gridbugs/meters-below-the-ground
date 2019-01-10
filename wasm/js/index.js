import { JsGrid, JsRenderer, installStyleSheet } from 'prototty-wasm-render-js';
import { ProtottyInput } from 'prototty-wasm-input-js';
import { JsByteStorage} from 'prototty-wasm-storage-js';
const wasm = import('../wasm_out/meters_wasm');

wasm.then(async wasm => {
    installStyleSheet({});
    let input = new ProtottyInput(new wasm.InputBuffer(), new wasm.InputBuffer());
    input.register();
    let js_grid = new JsGrid(app_node, 60, 45);
    let seed = parseInt(2**32 * Math.random());
    let storage = await JsByteStorage.make_async("fib");
    let app = new wasm.WebApp(seed, js_grid, storage);
    let previous_instant = Date.now();
    let tick = () => {
        let current_instant = Date.now();
        app.tick(input.swap_buffers(), current_instant - previous_instant);
        previous_instant = current_instant;
        requestAnimationFrame(tick);
    };
    tick()
});
