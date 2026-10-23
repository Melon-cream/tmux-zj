# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-23

### Added

- Zellij-style mode keys for tmux: `Ctrl+p` panes, `Ctrl+t` tabs, `Ctrl+h` pane
  swapping, `Ctrl+o` sessions. Arrow keys drive movement; the mode keys work
  from inside every mode, so modes can be hopped directly. `Esc`, `Enter` or
  `q` leaves, and actions that finish (split, close, fullscreen) leave the mode
  on their own.
- A status bar whose badge and hint line are read from tmux's own state
  (`client_key_table`, `pane_mode`), so the bar reverts however the mode was
  left and no shell command runs per redraw.
- Badges and hints for the two queued pane modes as well: `SESSIONS` for the
  session tree and `COPY` for copy mode.
- Mouse: drag selects and copies even over a pane that requested the mouse
  itself, with `set-clipboard` so the copy reaches the terminal over OSC 52.
- Session rename as a second pass over the session list (`Ctrl+o r`).
- `tmux-zj init` applies everything to the running server; `tmux-zj config`
  prints the generated config; `tmux-zj keys` prints the cheat sheet.
- One-line setup: `run-shell "tmux-zj init"`.
