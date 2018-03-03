#!/bin/bash

set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo clean --manifest-path=$DIR/../unix/Cargo.toml
cargo clean --manifest-path=$DIR/../wasm/Cargo.toml
cargo clean --manifest-path=$DIR/../glutin/Cargo.toml
cargo clean --manifest-path=$DIR/../prototty/Cargo.toml
cargo clean --manifest-path=$DIR/../meters/Cargo.toml
