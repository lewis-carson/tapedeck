
install PROJECT:
    cargo install --path {{PROJECT}}

install-all:
    just install record
    just install interleave
    just install accumulate

run PROJECT:
    cargo run --manifest-path {{PROJECT}}/Cargo.toml

reset DIR:
    rm -rf {{DIR}}
    mkdir -p {{DIR}}

record DIR:
    just reset {{DIR}}
    
    RUSTFLAGS="-Awarnings" cargo run -q --manifest-path record/Cargo.toml --release {{DIR}}

backtest DIR:
    RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./interleave/Cargo.toml {{DIR}} \
    | RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./strategy/Cargo.toml