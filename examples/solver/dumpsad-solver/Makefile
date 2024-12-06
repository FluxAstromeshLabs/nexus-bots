build:
	@cargo build --lib --target wasm32-unknown-unknown

build-release:
	@RUSTFLAGS='-C link-arg=-s' cargo build --lib --target wasm32-unknown-unknown --release

test:
	@cargo test --all -- --nocapture
