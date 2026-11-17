# Detects problems using rustfmt, clippy, and cargo check, and runs the linter and analyzer.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo check --workspace --locked

# Fixes linting problems automatically using clippy, cargo fix, and rustfmt.
fix:
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
    cargo fix --allow-dirty --allow-staged
    cargo fmt --all
