#!/bin/bash

set -e
set -x

#rm -rf output
#mkdir output

FUNC=zkmain

# Single test
#~/zkWasm/target/release/delphinus-cli -k 22 --function $FUNC --output ./output --param ./output --wasm ./pkg/output.wasm setup
#~/zkWasm/target/release/delphinus-cli -k 22 --phantom prepare_u64_vec --function $FUNC --output ./output --param ./output --wasm ./pkg/output.wasm dry-run --private 1:i64

~/zkWasm/target/release/delphinus-cli --phantom idx:9 -k 22  --function $FUNC --output ./output --param ./output --wasm ./pkg/output.wasm single-prove --private 2:i64

#~/zkWasm/target/release/delphinus-cli -k 22 --function $FUNC --output ./output --param ./output --wasm ./pkg/output.wasm single-prove --private 100:i64

# Perform host circuit proofs
#~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname poseidonhash --output output/
#~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname merkle --output output/
