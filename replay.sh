#!/bin/bash

mkdir -p data/

RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./interleave/Cargo.toml data/ \
|RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./strategy/Cargo.toml