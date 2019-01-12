#!/bin/bash

set -euxo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

BIN_NAME=meters
PROJECT_ROOT=$DIR/..
UNIX_CRATE=$PROJECT_ROOT/unix
WASM_CRATE=$PROJECT_ROOT/wasm
GLUTIN_CRATE=$PROJECT_ROOT/glutin
BUILD_DIR=$PROJECT_ROOT/build
UPLOAD_DIR=$PROJECT_ROOT/uploads
WEB_UPLOAD_DIR=$PROJECT_ROOT/web_uploads

if [ -z ${TRAVIS_OS_NAME+x} ]; then
    case `uname -s` in
        Linux)
            TRAVIS_OS_NAME=linux
            ;;
        Darwin)
            TRAVIS_OS_NAME=osx
            ;;
        *)
            echo "Unknown OS"
            exit 1
    esac
fi

case $TRAVIS_OS_NAME in
    linux)
        pyenv version 3
        PIP=pip
        PYTHON=python
        ;;
    osx)
        if ! which python3 > /dev/null; then
            brew install python3 || brew upgrade python
        fi
        PIP=pip3
        PYTHON=python3
        ;;
    local-archlinux)
        PIP=pip3
        PYTHON=python3
        ;;
esac

$PIP install --quiet --user sh toml

BUILD_PY="$PYTHON $DIR/build.py"
BUILD_PY_COMMON="$PYTHON $DIR/build.py --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$UPLOAD_DIR"

rm -rf $BUILD_DIR
rm -rf $UPLOAD_DIR
rm -rf $WEB_UPLOAD_DIR

case $TRAVIS_OS_NAME in
    linux)
        $BUILD_PY --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$WEB_UPLOAD_DIR \
            --os=unknown --frontend=wasm --crate-path=$WASM_CRATE
        $BUILD_PY_COMMON --os=linux --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        ;;
    osx)
        $BUILD_PY_COMMON --os=macos --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        ;;
    local-archlinux)
        $BUILD_PY --root-path=$PROJECT_ROOT --build-path=$BUILD_DIR --upload-path=$WEB_UPLOAD_DIR \
            --os=unknown --frontend=wasm --crate-path=$WASM_CRATE
        $BUILD_PY_COMMON --os=linux --frontend=unix --frontend=glutin --crate-path=$UNIX_CRATE --crate-path=$GLUTIN_CRATE
        ;;
esac
