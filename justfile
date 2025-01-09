export RUSTFLAGS := "-Zon-broken-pipe=kill -Awarnings"

install PROJECT:
    cargo install --path {{PROJECT}}

install-all:
    just install record
    just install interleave
    just install accumulate

run PROJECT *ARGS:
    cargo run -q --release --manifest-path {{PROJECT}}/Cargo.toml {{ARGS}}

reset DIR:
    rm -rf {{DIR}}
    mkdir -p {{DIR}}

record DIR:
    just reset {{DIR}}
    
    just run record {{DIR}}

replay-example DIR:
    just run interleave {{DIR}} \
    | just run accumulate \
    | just run replay-example