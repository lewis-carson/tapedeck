
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

replay-example DIR:
    RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./interleave/Cargo.toml {{DIR}} \
    | RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./accumulate/Cargo.toml \
    | RUSTFLAGS="-Awarnings" cargo run -q --release --manifest-path ./replay-example/Cargo.toml