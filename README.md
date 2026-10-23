# tmux-zj

Zellij-style modal keybindings for tmux. `Ctrl+p` opens pane actions, `Ctrl+t`
tab actions, and the status bar tells you where you are.

The status bar is read straight from tmux's own state (`client_key_table` and
`pane_mode`) instead of being rewritten by a helper process on each keypress.
Leave a mode by Esc, by an action that finishes, by detaching, or by switching
clients, and the bar is already correct: no stale `PANE` label, no `#()` shell
call on every redraw.

## Install

```sh
cargo install tmux-zj
```

tmux 3.1 or newer.

## Setup

Add one line to `~/.tmux.conf`:

```tmux
run-shell "tmux-zj init"
```

Then `tmux source-file ~/.tmux.conf`, or start tmux again. `tmux-zj config`
prints the same thing as a plain config file if you prefer to keep it inline.

## Keys

`Ctrl+<key>` enters a mode. `Esc`, `Enter` or `q` leaves it, and the same
`Ctrl+<key>` keys work inside every mode, so you can hop directly across.

| Key | Mode | Inside |
|---|---|---|
| `Ctrl+p` | PANE | arrows focus, `n` split right, `d` split down, `x` close, `f` fullscreen, `p` next pane, `Tab` last pane |
| `Ctrl+t` | TAB | arrows switch, `1`-`9` jump, `n` new, `x` close, `r` rename, `b` break out, `s` sync panes, `Tab` last tab |
| `Ctrl+h` | MOVE | arrows swap the pane with its neighbour, `n`/`p` next/prev |
| `Ctrl+o` | SESSION | `d` detach, `w` session tree, `r` rename a picked session, `[`/`]` previous/next |

`hjkl` still works as an alias for the arrows.

Direct actions, no mode needed: `Alt+n` new pane, `Alt+hjkl` focus (moving to
the previous/next tab at the edge), `Alt+=`/`Alt+-` widen/narrow, `Alt+i`/
`Alt+o` move the current tab left/right.

`tmux-zj keys` prints this list in the terminal.

## Mouse

Dragging selects text and copies it on release, including over a pane that
asked for the mouse itself (vim, less, htop). tmux's default rule hands those
drags to the pane; this sets `MouseDrag1Pane` to always select in tmux and
turns on `set-clipboard` so the copy reaches your terminal's clipboard over
OSC 52 (kitty, wezterm, iTerm2, alacritty, foot and friends).

The wheel scrolls the pane. `Ctrl+s` and `Ctrl+q` are left to the shell, so
your usual flow control and forward-search keep working.

## Painted modes

The badge on the left is one of `PANE`, `TAB`, `MOVE`, `SESSION`, `SESSIONS`
(the session tree) or `COPY` (copy mode), and the hints on the right change
with it. Normal mode shows no badge. Both are plain tmux formats, so they cost
nothing to render and cannot drift out of date.

The tool sets `status-style`, `status-left`, `status-right` and their lengths,
plus `mode-style` and `message-style` (tmux's own defaults are yellow; these are
tuned to the blue bar). Your window list and the tmux prefix are left alone.
Pass `--no-status` to keep your own bar.

`Ctrl+o r` opens the session list and renames whatever you pick: Enter, then type
the new name. tmux's tree mode has hardcoded keys and no key table, so nothing of
ours can run inside it and Escape does nothing there (use `q` to close it). The
name prompt itself does take Escape.

## Commands

| Command | Effect |
|---|---|
| `tmux-zj init` | apply keys, mouse and status bar to the running server |
| `tmux-zj config` | print the generated tmux config |
| `tmux-zj keys` | print the cheat sheet |
| `tmux-zj help` | usage |

| Option | Effect |
|---|---|
| `--no-status` | bindings only, leave the status bar alone |
| `--no-alt` | skip the `Alt+` direct actions |
| `--no-mouse` | leave mouse and clipboard settings alone |
| `-L NAME` | use this tmux socket (`tmux -L`) |

## Limits

- The `Alt+` keys are taken from the shell; use `--no-alt` if you need them back.
- With `--mouse on`, a drag over a mouse-aware pane always selects in tmux. Use
  `--no-mouse` if you would rather keep the pane's own mouse handling.
- Zellij features with no tmux equivalent are absent: floating panes, swap
  layouts, plugin panes. MOVE swaps panes in layout order, not geometrically.
- The status bar colour is `#3c6f99`; a terminal without 24-bit colour gets the
  nearest palette entry.
- The key tables are named `zj-pane`, `zj-tab`, `zj-move`, `zj-session` so they
  cannot collide with your own.

## Development

`cargo fmt --check`, `cargo clippy --release -- -D warnings` and the end-to-end
run in `e2e/` (a real tmux client on a pty, rendered and asserted) are the whole
check. `CONTRIBUTING.md` has the commands and the rules; `CHANGELOG.md` tracks
releases.

## License

MIT
