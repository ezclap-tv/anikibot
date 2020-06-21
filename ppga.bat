@echo off

cargo run --release --quiet --manifest-path .\ppga-script\ppga\Cargo.toml --features=build-binary -- %*
