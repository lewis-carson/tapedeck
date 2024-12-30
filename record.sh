#!/bin/bash

rm data/*
mkdir -p data/

RUSTFLAGS="-Awarnings" cargo run -q --manifest-path record/Cargo.toml --release data/