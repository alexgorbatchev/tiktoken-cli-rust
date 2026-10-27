run *args:
    cargo run --bin tiktoken -- {{args}}

run-ai *args:
    AGENT=1 cargo run --bin tiktoken -- {{args}}

build:
    cargo build --release --bin tiktoken
    mkdir -p bin
    cp target/release/tiktoken bin/tiktoken

test:
    cargo test --all-targets

lint:
    rustfmt --check --edition 2024 --config skip_children=true src/lib.rs src/main.rs tests/behavior.rs
    cargo clippy --all-targets -- -D warnings

coverage:
    cargo llvm-cov --all-targets --fail-under-lines 90

check: lint test coverage
