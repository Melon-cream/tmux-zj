# Contributing

Thanks for looking at tmux-zj. This document covers how to build, test and
contribute.

## Setup

The crate has no dependencies, so a stock Rust toolchain is all you need:

```sh
cargo build --release
cargo fmt --check
cargo clippy --release -- -D warnings
```

If you prefer not to install anything on the host, the dev container has tmux,
pyte, the musl target and clippy already:

```sh
docker build -t tmux-zj-dev docker
docker run --rm -v "$PWD":/w -w /w tmux-zj-dev \
  sh -c 'cargo fmt --check && cargo clippy --release -- -D warnings'
```

## Tests

There are no unit tests. The behaviour worth testing is what tmux draws on the
screen, so the check is one end-to-end run against a real tmux client on a pty:

```sh
python3 -m pip install pyte
ZJ_BIN=target/release/tmux-zj python3 e2e/e2e.py
```

It starts a throwaway server on its own socket, sends real key bytes, renders
the screen and asserts the badge, the hints and the mode transitions. It exits
non-zero on the first mismatch and prints what it saw.

## Ground rules

- **No runtime dependencies.** `src/` uses the standard library only. Keep it
  that way; the binary is installed on machines we do not control.
- **One source of truth.** Keys, labels and hints live in `src/model.rs`.
  Bindings and the status bar are generated from it, so adding a key means
  adding one entry, never editing the generated side by hand.
- **The display follows tmux, not the tool.** Status strings branch on
  `#{client_key_table}` / `#{pane_mode}` (both are plain tmux formats). Do not
  add a helper process that rewrites `status-*`, and never put `#(...)` in the
  status line.
- **Minimal diff.** Config knobs nobody asked for are a liability: tmux has
  enough of its own. Prefer deleting a mode over keeping a dead key table.
- **Explain tmux's limits in the README** rather than faking parity with
  zellij.

## Commits and pull requests

- Conventional commit subjects, one line, imperative: `feat:`, `fix:`,
  `docs:`, `test:`, `refactor:`, `chore:`. Under 72 characters.
- Open a pull request against `main`. CI (fmt, clippy, build, end-to-end) must
  pass.

## Reporting bugs

Open an issue with your `tmux -V`, your terminal, the output of `tmux-zj
config` (it is generated, so it shows exactly what is bound), and what you
pressed versus what happened.
