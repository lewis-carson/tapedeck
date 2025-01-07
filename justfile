export RUSTFLAGS := "-Awarnings"

install PROJECT:
    cargo install --path {{PROJECT}}

install-all:
    just install record
    just install interleave

run PROJECT:
    cargo run --manifest-path {{PROJECT}}/Cargo.toml

reset DIR:
    rm -rf {{DIR}}
    mkdir -p {{DIR}}

record DIR:
    just reset {{DIR}}
    
    cargo run -q --manifest-path record/Cargo.toml --release {{DIR}}

backtest DIR:
    cargo run -q --release --manifest-path ./interleave/Cargo.toml {{DIR}} \
    | cargo run -q --release --manifest-path ./strategy/Cargo.toml