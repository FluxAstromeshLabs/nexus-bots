build:
	@cargo build --lib --target wasm32-unknown-unknown
	@cosmwasm-check target/wasm32-unknown-unknown/debug/arbitrage_demo.wasm

build-release:
	@RUSTFLAGS='-C link-arg=-s --cfg target_arch="wasm32_1"' cargo build --lib --target wasm32-unknown-unknown --release
	@cosmwasm-check target/wasm32-unknown-unknown/release/arbitrage_demo.wasm

test:
	@cargo test --all -- --nocapture
