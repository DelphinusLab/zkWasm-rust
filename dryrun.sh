#!/bin/bash

set -e
set -x

FUNC=zkmain
~/zkWasm/target/release/delphinus-cli -k 22 --host standard --function $FUNC --output ./output --param ./params --wasm ./pkg/output.wasm dry-run
