build:
	wasm-pack build --release --out-name rust-sdk-test.wasm --out-dir pkg
	wasm-opt -Oz -o pkg/test.wasm pkg/rust-sdk-test.wasm

trace:
	wasm-interp pkg/test.wasm --run-all-exports  --trace > trace.log
	wc -l trace.log
