# onchain-strategy

## install wasm target
rustup target add wasm32-unknown-unknown

## build strategy
make build
make build-release

## deploy & trigger
output binary at `target/wasm32-unknown-unknown/bank_strategy.wasm`
deploy using `sdk-go/examples/chain/21_MsgConfigStrategy`
trigger using `sdk-go/examples/chain/22_MsgTriggerStrategy`
