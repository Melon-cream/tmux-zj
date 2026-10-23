#!/usr/bin/env python3
"""End-to-end check: a real tmux client on a pty, keys sent as bytes, the status
line rendered by pyte and asserted as the user would see it."""
import fcntl
import os
import pty
import select
import struct
import subprocess
import sys
import termios
import time

import pyte

SOCK = "zje2e"
BIN = os.environ.get("ZJ_BIN", "target/release/tmux-zj")
W, H = 110, 14

failures = []


def tmux(*args, check=False):
    p = subprocess.run(["tmux", "-L", SOCK, *args], capture_output=True, text=True)
    if check and p.returncode != 0:
        raise RuntimeError(f"tmux {args}: {p.stderr.strip()}")
    return p.stdout.strip()


def expect(label, actual, wanted):
    ok = wanted in actual if isinstance(wanted, str) else actual == wanted
    print(f"{'ok  ' if ok else 'FAIL'} {label}")
    if not ok:
        print(f"     wanted: {wanted!r}\n     actual: {actual!r}")
        failures.append(label)


def reject(label, actual, unwanted):
    ok = unwanted not in actual
    print(f"{'ok  ' if ok else 'FAIL'} {label}")
    if not ok:
        print(f"     must not contain: {unwanted!r}\n     actual: {actual!r}")
        failures.append(label)


def main():
    subprocess.run(["tmux", "-L", SOCK, "kill-server"], capture_output=True)
    subprocess.run(
        ["tmux", "-L", SOCK, "-f", "/dev/null", "new", "-d", "-s", "alpha", "-x", str(W),
         "-y", str(H), "printf 'hello world\\nsecond line\\n'; sleep 3600"],
        check=True,
    )
    init = subprocess.run([BIN, "init", "-L", SOCK], capture_output=True, text=True)
    expect("init exits 0", init.returncode, 0)
    if init.returncode != 0:
        print(init.stderr)
        return 1

    cfg = tmux("show", "-g", "status-left") + tmux("show", "-g", "status-right")
    reject("status has no per-render subprocess", cfg, "#(")
    expect("status-left reads client_key_table", cfg, "client_key_table")
    expect("status-left reads pane_mode", cfg, "pane_mode")
    expect("bar colour applied", tmux("show", "-gv", "status-style"), "3c6f99")
    expect("no Ctrl-S binding", tmux("list-keys", "-T", "root", "C-s"), "")
    expect("no Ctrl-Q binding", tmux("list-keys", "-T", "root", "C-q"), "")
    expect("no Ctrl-N binding", tmux("list-keys", "-T", "root", "C-n"), "")
    expect("no resize table", tmux("list-keys", "-T", "zj-resize"), "")

    pid, fd = pty.fork()
    if pid == 0:
        attrs = termios.tcgetattr(0)
        attrs[0] &= ~(termios.IXON | termios.IXOFF)
        termios.tcsetattr(0, termios.TCSANOW, attrs)
        os.environ["TERM"] = "xterm-256color"
        os.execvp("tmux", ["tmux", "-L", SOCK, "-f", "/dev/null", "attach", "-t", "alpha"])

    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", H, W, 0, 0))
    screen = pyte.Screen(W, H)
    stream = pyte.ByteStream(screen)
    # tmux probes terminal capabilities with CSI 5 n. pyte's ByteStream passes
    # `private=True`, but Screen.report_device_status accepts only `mode`.
    stream.listener.report_device_status = lambda mode=None, **kwargs: None

    def pump(seconds=0.6):
        end = time.time() + seconds
        while time.time() < end:
            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                try:
                    data = os.read(fd, 65536)
                except OSError:
                    return
                if not data:
                    return
                stream.feed(data)

    def keys(data, wait=0.6):
        os.write(fd, data)
        pump(wait)

    def row():
        return screen.display[H - 1].rstrip()

    def cell_bg(x):
        return list(screen.buffer[H - 1].values())[x].bg

    def screen_text():
        return "\n".join(screen.display[i].rstrip() for i in range(H))

    def table():
        return tmux("display", "-p", "-F", "#{client_key_table}")

    def pane_mode():
        return tmux("display", "-p", "-F", "#{pane_mode}")

    def banner():
        return "".join(c.data for c in list(screen.buffer[H - 1].values())[:14])

    def badge_padded(name):
        """The colour block must cover a space on both sides of the badge text."""
        cells = list(screen.buffer[H - 1].values())[:14]
        text = "".join(c.data for c in cells)
        i = text.find(name)
        if i <= 0:
            return False
        return cells[i - 1].bg == cells[i].bg and cells[i + len(name)].bg == cells[i].bg

    pump(1.5)
    print(f"\nroot row: {row()!r}")
    print(f"root banner: {banner()!r}\n")
    expect("bar background configured", tmux("show", "-gv", "status-style"), "3c6f99")
    expect("bar background painted", cell_bg(40) != "default", True)
    expect("window list visible", row(), "0:")
    expect("root: menu", row(), "^p pane")
    reject("root: no NORMAL word", row(), "NORMAL")
    expect("root: table", table(), "root")

    keys(b"\x10")  # C-p
    expect("C-p: table", table(), "zj-pane")
    expect("C-p: badge", banner(), "PANE")
    expect("C-p: arrow hints", row(), "←↑↓→")
    expect("C-p: hints", row(), "split")
    expect("PANE badge is padded", badge_padded("PANE"), True)
    reject("C-p: menu replaced", row(), "^p pane")

    keys(b"\x14")  # C-t from inside pane mode
    expect("C-t from pane: table", table(), "zj-tab")
    expect("C-t from pane: badge", banner(), "TAB")
    expect("C-t from pane: hints", row(), "goto")

    before = tmux("display", "-p", "-F", "#{session_windows}")
    keys(b"n")  # tab mode: new window, the mode ends by itself
    after = tmux("display", "-p", "-F", "#{session_windows}")
    expect("n created a window", f"{before}->{after}", f"{before}->{int(before) + 1}")
    expect("n left the mode", table(), "root")
    expect("n reverted the banner", banner(), "")
    expect("n reverted the hints", row(), "^p pane")

    keys(b"\x14")  # C-t with two windows now
    win_before = tmux("display", "-p", "-F", "#{window_index}")
    keys(b"\x1bOD")  # Left -> previous window
    win_after = tmux("display", "-p", "-F", "#{window_index}")
    expect("tab Left switched window", win_before != win_after, True)
    expect("tab Left kept the mode", table(), "zj-tab")
    expect("tab Left kept the badge", banner(), "TAB")
    keys(b"\x1b")

    # arrows drive pane focus
    keys(b"\x10")
    keys(b"n")  # split
    keys(b"\x10")
    left = tmux("display", "-p", "-F", "#{pane_id}")
    keys(b"\x1bOC")  # Right (application cursor mode)
    right = tmux("display", "-p", "-F", "#{pane_id}")
    expect("arrow moved focus", left != right, True)
    expect("arrow kept the mode", table(), "zj-pane")
    keys(b"\x1b")  # Esc
    expect("Esc leaves a mode", table(), "root")

    # wheel scroll -> copy mode, and only the COPY badge
    keys(b"\x1b[<64;5;5M", 0.8)
    expect("wheel: copy mode", pane_mode(), "copy-mode")
    expect("wheel: COPY badge", banner(), "COPY")
    expect("wheel: hints", row(), "select")
    keys(b"q", 0.8)
    expect("q leaves copy mode", pane_mode(), "")
    expect("q clears the badge", banner(), "")

    # session mode: tree shows SESSIONS, not COPY
    tmux("new-session", "-d", "-s", "beta", "sleep 3600")
    keys(b"\x0f")
    expect("C-o: table", table(), "zj-session")
    expect("C-o: badge", banner(), "SESSION")
    expect("C-o: hints", row(), "detach")
    keys(b"w", 1.0)
    expect("w: tree mode", pane_mode(), "tree-mode")
    expect("w: SESSIONS badge", banner(), "SESSIONS")
    reject("w: not COPY", banner(), "COPY")
    expect("w: tree hints", row(), "open")
    keys(b"q", 0.8)
    expect("tree closed", pane_mode(), "")

    # session mode: rename is a second pass over the session list
    keys(b"\x0f")
    keys(b"r", 1.2)
    expect("r: rename list opens", pane_mode(), "tree-mode")
    expect("r: list is the session list", table(), "root")
    keys(b"\r", 1.0)  # choose the first entry
    expect("r: prompt opens", screen_text(), "rename =")
    keys(b"gamma\r", 1.0)
    names = tmux("list-sessions", "-F", "#{session_name}").split()
    expect("rename applied to the chosen session", "gamma" in names, True)
    expect("session count unchanged", len(names), 2)

    # mouse drag over a pane that grabbed the mouse itself
    mouse_win = tmux("new-window", "-d", "-P", "-F", "#{window_id}", "-t", "gamma",
                     "printf 'HELLO_MOUSE\\n\\033[?1000h\\033[?1006h'; sleep 3600")
    tmux("select-window", "-t", mouse_win)
    pump(1.2)
    print(f"     mouse pane row1: {screen.display[0][:20]!r}")
    tmux("set-buffer", "")
    for seq in (b"\x1b[<0;1;1M", b"\x1b[<32;5;1M", b"\x1b[<0;5;1m"):
        os.write(fd, seq)
        time.sleep(0.12)
    pump(0.8)
    expect("drag copied from a mouse-grabbing pane", tmux("show-buffer"), "HELL")

    # an action that cannot run must not drop the mode
    tmux("new-session", "-d", "-s", "solo", "sleep 3600")
    tmux("switch-client", "-t", "solo")
    pump(0.8)
    keys(b"\x14")
    keys(b"\x1bOD", 0.9)  # previous window with a single window
    expect("edge no-op keeps the mode", table(), "zj-tab")
    keys(b"\x1b")
    tmux("switch-client", "-t", "gamma")
    pump(0.6)

    # the bar follows tmux, not anything the tool cached
    keys(b"\x10")
    tmux("switch-client", "-T", "root")
    tmux("refresh-client")
    pump()
    expect("external table change clears the badge", banner(), "")

    again = subprocess.run([BIN, "init", "-L", SOCK], capture_output=True, text=True)
    expect("second init exits 0", again.returncode, 0)
    pump()
    expect("second init leaves the table", table(), "root")

    sheet = subprocess.run([BIN, "keys"], capture_output=True, text=True).stdout
    expect("keys sheet lists modes", sheet, "SESSION (C-o)")
    expect("keys sheet lists mouse", sheet, "drag")

    print("\nfinal row:", repr(row()))
    os.close(fd)
    subprocess.run(["tmux", "-L", SOCK, "kill-server"], capture_output=True)
    try:
        os.kill(pid, 9)
    except ProcessLookupError:
        pass

    print(f"\n{len(failures)} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
