#!/bin/bash

set -e
set -x

HOSTCLI=$HOME/zkWasm-host-circuits/target/release/zkwasm-host-circuits-prover

$HOSTCLI --input ./output/traces/external_host_table.$1.json --opname poseidonhash --output output/ --param params
$HOSTCLI --input ./output/traces/external_host_table.$1.json --opname merkle --output output/ --param params
$HOSTCLI --input ./output/traces/external_host_table.$1.json --opname jubjubsum --output output/ --param params
