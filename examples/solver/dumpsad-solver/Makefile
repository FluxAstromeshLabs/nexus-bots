build:
	@cargo build --lib --target wasm32-unknown-unknown
	@cosmwasm-check target/wasm32-unknown-unknown/debug/dumpsad_solver.wasm

build-release:
	@RUSTFLAGS='-C link-arg=-s' cargo build --lib --target wasm32-unknown-unknown --release
	@cosmwasm-check target/wasm32-unknown-unknown/release/dumpsad_solver.wasm

test:
	@cargo test --all -- --nocapture
