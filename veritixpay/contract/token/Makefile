default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

build-logs:
	cargo build --profile release-with-logs --target wasm32-unknown-unknown
	@ls -l ../../target/wasm32-unknown-unknown/release-with-logs/*.wasm

test-logs:
	cargo test --profile release-with-logs

fmt:
	cargo fmt --all

clean:
	cargo clean
