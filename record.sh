#!/bin/bash

rm data/*
mkdir -p data/

cargo run --manifest-path record/Cargo.toml --release data/