check:
    cargo clippy -- -D warnings

format:
    cargo fmt

test:
    cargo test -- --skip stardew
    cargo test stardew -- --test-threads=1 # avoid flaky tests