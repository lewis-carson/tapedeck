export RUSTFLAGS := "-Awarnings"
export RUST_BACKTRACE := "1"

@install PROJECT:
    cargo install - {{PROJECT}}

@install-all:
    just install record
    just install interleave
    just install accumulate

@run-release PROJECT *ARGS:
    cargo run -q --release -p {{PROJECT}} {{ARGS}}

@run PROJECT *ARGS:
    cargo run -q -p {{PROJECT}} {{ARGS}}

@reset DIR:
    rm -rf {{DIR}}
    mkdir -p {{DIR}}

@record DIR:
    just reset {{DIR}}
    
    just run-release record {{DIR}}
