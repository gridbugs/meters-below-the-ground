#!/bin/bash

CRATE="punchcards_wasm"

set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

pushd $DIR

if [[ "$1" == '--with-npm-install' ]]; then
    npm install
fi

cargo build \
    --target=wasm32-unknown-unknown --release

wasm-gc \
    target/wasm32-unknown-unknown/release/$CRATE.wasm \
    dist/$CRATE.wasm

npx webpack

popd
