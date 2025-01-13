export RUSTFLAGS := "-Awarnings"
export RUST_BACKTRACE := "1"

@install PROJECT:
    cargo install --path {{PROJECT}}

@install-all:
    just install record
    just install interleave
    just install accumulate

@run-release PROJECT *ARGS:
    cargo run -q --release --manifest-path {{PROJECT}}/Cargo.toml {{ARGS}}

@run PROJECT *ARGS:
    cargo run -q --manifest-path {{PROJECT}}/Cargo.toml {{ARGS}}

@reset DIR:
    rm -rf {{DIR}}
    mkdir -p {{DIR}}

@record DIR:
    just reset {{DIR}}
    
    just run-release record {{DIR}}

@watch +FILES:
    python3 watch/main.py {{FILES}}