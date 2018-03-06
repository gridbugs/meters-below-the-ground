#!/bin/bash

set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

BIN_NAME=meters
PROJECT_ROOT=$DIR/..
UNIX_CRATE=$PROJECT_ROOT/unix
WASM_CRATE=$PROJECT_ROOT/wasm
GLUTIN_CRATE=$PROJECT_ROOT/glutin
BUILD_DIR=$PROJECT_ROOT/build
UPLOAD_DIR=$PROJECT_ROOT/uploads
WEB_UPLOAD_DIR=$PROJECT_ROOT/web_uploads

source $DIR/deps.sh
BUILD_PY="$PYTHON $DIR/build.py"
BUILD_PY_COMMON="$PYTHON $DIR/build.py --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$UPLOAD_DIR"

rm -rf $BUILD_DIR
rm -rf $UPLOAD_DIR
rm -rf $WEB_UPLOAD_DIR

case $TRAVIS_OS_NAME in
    linux)
        if [[ "$TRAVIS_RUST_VERSION" == "beta" ]] || [[ "$TRAVIS_RUST_VERSION" == "nightly" ]]; then
            $BUILD_PY --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$WEB_UPLOAD_DIR \
                --os=unknown --frontend=wasm --crate-path=$WASM_CRATE
        fi
        if [[ "$TRAVIS_RUST_VERSION" != "beta" ]]; then
            # This can take a long time, and it's important that beta succeeds and uploads the wasm build artifact
            $BUILD_PY_COMMON --os=linux --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        fi
        ;;
    osx)
        $BUILD_PY_COMMON --os=macos --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        ;;
    local-archlinux)
        $BUILD_PY_COMMON --os=linux --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        $BUILD_PY --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$WEB_UPLOAD_DIR \
            --os=unknown --frontend=wasm --crate-path=$WASM_CRATE
        ;;
esac
