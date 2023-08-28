#!/bin/bash

set -e
set -x

#rm -rf output
#mkdir output

# Single test
~/zkWasm/target/release/delphinus-cli -k 22 --function zkmain --output ./output --wasm ./pkg/output.wasm setup
~/zkWasm/target/release/delphinus-cli -k 22 --function zkmain --output ./output --wasm ./pkg/output.wasm dry-run
~/zkWasm/target/release/delphinus-cli -k 22 --function zkmain --output ./output --wasm ./pkg/output.wasm aggregate-prove

# Perform host circuit proofs
~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname poseidonhash --output output/
~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname merkle --output output/
