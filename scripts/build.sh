#!/bin/bash

set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

BIN_NAME=punchcards
PROJECT_ROOT=$DIR/..
UNIX_CRATE=$PROJECT_ROOT/unix
WASM_CRATE=$PROJECT_ROOT/wasm
GLUTIN_CRATE=$PROJECT_ROOT/glutin
BUILD_DIR=$PROJECT_ROOT/build

BUILD_PY=$DIR/build.py

source $DIR/deps.sh

$PYTHON $BUILD_PY
