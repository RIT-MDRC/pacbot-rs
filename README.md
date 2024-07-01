# Pacbot-rs

Code for the Harvard Pacbot competition reimplemented in Rust. Based on commit `ef1c4614d8bdaa167d0e5f9b965613efdfaa9112` of the Go Pacbot repo.

This code should build for all the following targets/features:

```bash
cargo build --features std
cargo build --no-default-features

cargo build --target wasm32-unknown-unknown --features std,wasm
cargo build --no-default-features --target thumbv6m-none-eabi
```