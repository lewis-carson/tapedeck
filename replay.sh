#!/bin/bash

mkdir -p data/

cargo run --release --manifest-path ./interleave/Cargo.toml data/ | cargo run --release --manifest-path ./replay/Cargo.toml