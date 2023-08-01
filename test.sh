#!/bin/bash

set -e
set -x

#rm -rf output
#mkdir output

# Single test
#~/zkWasm/target/release/cli  -k 18 --function zkmain --output ./output --wasm ./pkg/output.wasm setup
#~/zkWasm/target/release/cli  -k 18 --function zkmain --output ./output --wasm ./pkg/output.wasm dry-run
~/zkWasm/target/release/cli  -k 18 --function zkmain --output ./output --wasm ./pkg/output.wasm single-prove
