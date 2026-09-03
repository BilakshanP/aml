# Contributing to AML

Thanks for contributing! This guide covers the local development setup.

## Prerequisites

- A recent stable Rust toolchain (`rustup toolchain install stable`).
- `rustfmt` and `clippy` components (`rustup component add rustfmt clippy`).

## Running tests

Always test with **all features enabled**, so feature-gated code (e.g. `styler`,
`quote`) is compiled and exercised:

```bash
cargo test --workspace --all-features
```

A convenience alias is defined in [`.cargo/config.toml`](.cargo/config.toml):

```bash
cargo t   # == cargo test --all-features
```

> Plain `cargo test` only runs whatever the default feature graph happens to
> unify on, which can silently skip feature-gated tests. Prefer `--all-features`.

## Git hooks

This repository ships version-controlled git hooks in [`.githooks/`](.githooks/).
They are **not** installed automatically — enable them once per clone by pointing
git at that directory:

```bash
git config core.hooksPath .githooks
```

This is preferred over copying or symlinking into `.git/hooks`, because the hooks
stay tracked in the repository and update with `git pull`.

### `pre-push`

Runs `cargo test --workspace --all-features` before every push and aborts the
push if any test fails. To bypass it in an emergency (avoid as a habit):

```bash
git push --no-verify
```

## Before opening a pull request

Make sure the same checks CI runs pass locally:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets
cargo test --workspace --all-features
```

CI (see [`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs these on
every push and pull request against `main`.
