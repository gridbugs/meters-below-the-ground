#!/bin/bash
set -euxo pipefail
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

pushd $DIR
./build_wasm.sh release
WEBPACK_MODE=production npx webpack
popd
