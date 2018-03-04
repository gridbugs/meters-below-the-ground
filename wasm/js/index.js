import $ from 'jquery';
import prototty from 'prototty-terminal-js';

let quit_after_save = false;

const config = {
    quit: () => {
        quit_after_save = true;
    },
    cell_style: {
        "font-family": "PxPlus_IBM_CGAthin",
        "font-size": "16px",
        "line-height": "0px",
    },
    bold_style: {
        "font-family": "PxPlus_IBM_CGA",
    },
    before_store: () => console.log("Saving..."),
    after_store: () => {
        console.log("Saved!");
        if (quit_after_save) {
            console.log("Quitting...");
            open(location, "_self").close();
        }
    }
};

$(() => {
    console.log("Loading wasm program...");
    prototty.loadProtottyApp("meters_wasm.wasm", 60, 34, protottyTerminal, config).then(app => {
        console.log("Loaded wasm program!");
        app.start();
    });
});
