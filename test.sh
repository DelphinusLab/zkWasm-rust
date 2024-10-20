#!/bin/bash

set -e
set -x

rm -rf output
mkdir output

FUNC=zkmain
CLI=$HOME/zkWasm/target/release/zkwasm-cli

# Single test
$CLI --params ./params rustsdk setup --host standard
$CLI --params ./params rustsdk dry-run --wasm ./pkg/output.wasm --output ./output --private 2:i64
$CLI --params ./params rustsdk prove --wasm ./pkg/output.wasm --output ./output --private 2:i64
