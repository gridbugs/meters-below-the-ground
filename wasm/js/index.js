'use strict';

import { Context, rngSeed } from 'prototty';
const wasm = import('../wasm_out/app');

wasm.then(async wasm => {
    let config = {
        WasmInputBufferType: wasm.InputBuffer,
        node: app_node,
        grid_width: 60,
        grid_height: 45,
        font_family: "PxPlus_IBM_CGA",
        font_size: "16px",
        cell_width_px: 16,
        cell_height_px: 18,
    };
    let context = await (new Context(config)).with_storage("meters");
    let app = new wasm.WebApp(rngSeed(), context.grid(), context.storage());
    context.run_animation((input_buffer, period) => app.tick(input_buffer, period));
});
