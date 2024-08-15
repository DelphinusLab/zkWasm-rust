#!/bin/bash

set -e
set -x

rm -rf output
mkdir output

FUNC=zkmain

# Single test
~/zkWasm/target/release/zkwasm-cli --params ./params rustsdk setup --host standard
~/zkWasm/target/release/zkwasm-cli --params ./params rustsdk dry-run --wasm ./pkg/output.wasm --output ./output --private 2:i64
~/zkWasm/target/release/zkwasm-cli --params ./params rustsdk prove --wasm ./pkg/output.wasm --output ./output --private 2:i64

#~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname poseidonhash --output output/ --param params
#~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname merkle --output output/ --param params
#~/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover --input external_host_table.json --opname jubjubsum --output output/ --param params
