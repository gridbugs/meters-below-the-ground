#!/bin/bash
set -euxo pipefail
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

NAME=$(cat Cargo.toml | grep 'name *=' | head -n1 | cut -d= -f2 | xargs echo)
WASM_FILE=$NAME.wasm

if [ "$#" -ne 1 ]; then
    echo "Usage $0 (release|debug)"
    exit 1
fi
MODE=$1

TOP_LEVEL_DIR="$DIR/../"
WASM_DIR_RAW=$TOP_LEVEL_DIR/target/wasm32-unknown-unknown/$MODE
WASM_DIR=wasm_out

case $MODE in
    release)
        CARGO_ARGS="--release"
        ;;
    debug)
        CARGO_ARGS=""
        ;;
    *)
esac
mkdir -p $WASM_DIR
cargo build --target=wasm32-unknown-unknown $CARGO_ARGS
wasm-bindgen $WASM_DIR_RAW/$WASM_FILE --out-dir $WASM_DIR --out-name app
