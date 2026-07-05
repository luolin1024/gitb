# Contributing to gitb

Thanks for your interest in improving gitb! This is a short guide.

## Development setup

```bash
git clone https://github.com/USER/gitb.git
cd gitb
cargo build
cargo test
```

## Before submitting a PR

All three must pass locally:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs these on Ubuntu, macOS, and Windows. Match the existing code style — `cargo fmt` is the source of truth.

## Filing issues

- **Bug**: include `gitb --version`, OS, and a minimal reproduction.
- **Feature**: describe the use case first, not just the command you want.

## Commit messages

Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`.

## License

By contributing, you agree your changes are licensed under MIT.
