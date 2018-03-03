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
        $BUILD_PY_COMMON --os=linux --frontend=unix --crate-path=$UNIX_CRATE
        $BUILD_PY_COMMON --os=linux --frontend=glutin --crate-path=$GLUTIN_CRATE
        ;;
    osx)
        $BUILD_PY_COMMON --os=macos --frontend=unix --crate-path=$UNIX_CRATE
        $BUILD_PY_COMMON --os=macos --frontend=glutin --crate-path=$GLUTIN_CRATE
        ;;
    local-archlinux)
        $BUILD_PY_COMMON --os=linux --frontend=unix --crate-path=$UNIX_CRATE
        $BUILD_PY_COMMON --os=linux --frontend=glutin --crate-path=$GLUTIN_CRATE
        $BUILD_PY --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$WEB_UPLOAD_DIR \
            --os=unknown --frontend=wasm --crate-path=$WASM_CRATE
        ;;
esac
