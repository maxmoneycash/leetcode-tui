# Contributing

Thanks for taking the time to contribute.

## Getting set up

```sh
git clone https://github.com/maxmoneycash/leetcode-tui
cd leetcode-tui
cargo build
```

### Minimum supported Rust version

**Rust 1.70.** CI builds on both `stable` and `1.70`, so please check your
change against the older toolchain before opening a pull request:

```sh
rustup toolchain install 1.70 --profile minimal
cargo +1.70 build && cargo +1.70 test
```

This matters more than it looks. Anything stabilised after 1.70 will compile
happily on your machine and fail in CI — `usize::div_ceil` (1.73) has already
caught us once. Clippy on a modern toolchain will sometimes *suggest* such a
replacement; if you take the suggestion, guard it with an `#[allow]` and a
comment, as `src/grind/chart.rs` does.

The `Cargo.lock` in this repo is deliberately kept in **v3 format** so cargo
1.70 can parse it. If you need to change a dependency, run the update with the
older toolchain (`cargo +1.70 update -p <crate>`) so the format is preserved.

## Running it

The full TUI needs LeetCode cookies. Run `leetui` once to generate the config,
then put your `LEETCODE_SESSION` and `csrftoken` in it — see the README.

Grind mode needs none of that and is the quickest way to exercise the UI:

```sh
cargo run -- grind
```

## Before you open a pull request

CI runs all of these, so run them locally first:

```sh
cargo fmt -- --check
cargo clippy --workspace --all-features
cargo test
cargo build --release
```

There is also a pre-commit config (`.pre-commit-config.yaml`) that wires up
fmt, check and clippy if you use [pre-commit](https://pre-commit.com/).

### Installing locally

Use `--locked`:

```sh
cargo install --path "." --force --locked
```

Without it, cargo re-resolves the dependency graph from scratch and currently
picks a `sea-orm-cli` / `regex` pair that does not compile. This is unrelated
to anything in this repo's own source.

## Testing terminal UI changes

The UI is hard to unit test, so keep the logic separable from the rendering —
the grind module is laid out this way on purpose:

- `engine.rs` / `candles.rs` hold the pure logic and carry the unit tests.
  Time is passed *in* as a parameter rather than read from the clock, which is
  what makes them deterministic and testable.
- `ui.rs` / `chart.rs` only draw.

To drive the real binary non-interactively, run it under tmux:

```sh
tmux new-session -d -s tui -x 120 -y 36 'cargo run -- grind'
tmux send-keys -t tui Enter
tmux capture-pane -t tui -p     # read the screen back
tmux kill-session -t tui
```

Please include what you ran and what you saw in the pull request description.

## Commit messages

Conventional-commit style prefixes (`feat:`, `fix:`, `refactor:`, `chore:`)
are used throughout the history. Add a `CHANGELOG.md` entry under
`## [Unreleased]` for anything user-visible.

## Reporting bugs

Open an issue with your OS, terminal emulator, `rustc --version`, and the
steps to reproduce. Terminal rendering bugs are much easier to act on with a
`tmux capture-pane -p` dump or a screenshot attached.
