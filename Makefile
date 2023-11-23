build:
	wasm-pack build --release --out-name rust-sdk-test.wasm --out-dir pkg --features witness
	wasm-opt -Oz -o pkg/output.wasm pkg/rust-sdk-test.wasm
	cp pkg/rust-sdk-test.wasm pkg/output.wasm

trace:
	wasm-interp pkg/output.wasm --run-all-exports  --trace > trace.log
	wc -l trace.log

clean:
	rm -rf pkg
