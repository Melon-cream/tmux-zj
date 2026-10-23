//! The single source of truth: every mode, every key, every hint label.
//!
//! Bindings and the status bar are generated from this table, so the hints on
//! screen cannot drift away from what the keys actually do.

pub const ROOT_TABLE: &str = "root";

#[derive(Clone, Copy)]
pub struct Hint {
    pub keys: &'static str,
    pub label: &'static str,
}

pub struct Binding {
    /// tmux key names. Alternate keys share one hint.
    pub keys: &'static [&'static str],
    /// Shown in the mode's hint bar. `None` for alternate keys.
    pub hint: Option<Hint>,
    /// tmux command, run as one argument.
    pub cmd: &'static str,
    /// Drop back to the root table after the command (zellij's "action ends the mode").
    pub leave: bool,
}

pub struct Mode {
    pub table: &'static str,
    pub name: &'static str,
    /// Lower-case label used in the root hint bar.
    pub menu: &'static str,
    /// Badge background, picked to sit on the blue status bar.
    pub colour: &'static str,
    /// The key that enters this mode from anywhere.
    pub enter: &'static str,
    pub bindings: &'static [Binding],
}

const fn bind(
    keys: &'static [&'static str],
    hint: Option<Hint>,
    cmd: &'static str,
    leave: bool,
) -> Binding {
    Binding {
        keys,
        hint,
        cmd,
        leave,
    }
}

const fn hint(keys: &'static str, label: &'static str) -> Option<Hint> {
    Some(Hint { keys, label })
}

pub static PANE: Mode = Mode {
    table: "zj-pane",
    name: "PANE",
    menu: "pane",
    colour: "#ffb454",
    enter: "C-p",
    bindings: &[
        bind(
            &["Left", "h"],
            hint("←↑↓→", "focus"),
            "select-pane -L",
            false,
        ),
        bind(&["Down", "j"], None, "select-pane -D", false),
        bind(&["Up", "k"], None, "select-pane -U", false),
        bind(&["Right", "l"], None, "select-pane -R", false),
        bind(
            &["n"],
            hint("n", "split"),
            "split-window -h -c \"#{pane_current_path}\"",
            true,
        ),
        bind(
            &["d"],
            hint("d", "down"),
            "split-window -v -c \"#{pane_current_path}\"",
            true,
        ),
        bind(
            &["x"],
            hint("x", "close"),
            "confirm-before -p \"close pane? (y/n)\" kill-pane",
            true,
        ),
        bind(&["f"], hint("f", "full"), "resize-pane -Z", true),
        bind(&["p"], hint("p", "next"), "select-pane -t :.+", false),
        bind(&["Tab"], None, "select-pane -l", false),
    ],
};

pub static TAB: Mode = Mode {
    table: "zj-tab",
    name: "TAB",
    menu: "tab",
    colour: "#7ee787",
    enter: "C-t",
    bindings: &[
        bind(&["Left", "h"], hint("←→", "tabs"), "previous-window", false),
        bind(&["Right", "l"], None, "next-window", false),
        bind(&["Up", "k"], None, "previous-window", false),
        bind(&["Down", "j"], None, "next-window", false),
        bind(&["1"], hint("1-9", "goto"), "select-window -t 1", true),
        bind(&["2"], None, "select-window -t 2", true),
        bind(&["3"], None, "select-window -t 3", true),
        bind(&["4"], None, "select-window -t 4", true),
        bind(&["5"], None, "select-window -t 5", true),
        bind(&["6"], None, "select-window -t 6", true),
        bind(&["7"], None, "select-window -t 7", true),
        bind(&["8"], None, "select-window -t 8", true),
        bind(&["9"], None, "select-window -t 9", true),
        bind(
            &["n"],
            hint("n", "new"),
            "new-window -c \"#{pane_current_path}\"",
            true,
        ),
        bind(
            &["x"],
            hint("x", "close"),
            "confirm-before -p \"close tab? (y/n)\" kill-window",
            true,
        ),
        bind(
            &["r"],
            hint("r", "rename"),
            "command-prompt -I \"#W\" \"rename-window '%%'\"",
            true,
        ),
        bind(&["b"], None, "break-pane", true),
        bind(
            &["s"],
            hint("s", "sync"),
            "if-shell -F \"#{synchronize-panes}\" \"set -w synchronize-panes off\" \"set -w synchronize-panes on\"",
            true,
        ),
        bind(&["Tab"], None, "last-window", false),
    ],
};

pub static MOVE: Mode = Mode {
    table: "zj-move",
    name: "MOVE",
    menu: "move",
    colour: "#d2a8ff",
    enter: "C-h",
    bindings: &[
        bind(
            &["Left", "h", "Up", "k"],
            hint("←↑↓→", "swap"),
            "swap-pane -d -U",
            false,
        ),
        bind(&["Right", "l", "Down", "j"], None, "swap-pane -d -D", false),
        bind(&["n", "Tab"], hint("n", "next"), "swap-pane -d -D", false),
        bind(&["p"], hint("p", "prev"), "swap-pane -d -U", false),
    ],
};

pub static SESSION: Mode = Mode {
    table: "zj-session",
    name: "SESSION",
    menu: "session",
    colour: "#ff9e9e",
    enter: "C-o",
    bindings: &[
        bind(&["d"], hint("d", "detach"), "detach-client", true),
        bind(&["w"], hint("w", "tree"), "choose-tree -Zs", true),
        // choose-tree has no key table and no Escape handling, so rename is a
        // second pass over the same list: pick the session, then type the name.
        // choose-tree replaces its first %% with the target (which carries a
        // leading =), so the prompt label absorbs that one and the second %%
        // is left for command-prompt itself.
        bind(
            &["r"],
            hint("r", "rename"),
            "choose-tree -Zs -F \"#{session_name}\" \
             'command-prompt -p \"rename %% to\" \"rename-session -t %1 %%\"'",
            true,
        ),
        bind(&["["], hint("[ ]", "sessions"), "switch-client -p", true),
        bind(&["]"], None, "switch-client -n", true),
    ],
};

pub static MODES: &[&Mode] = &[&PANE, &TAB, &MOVE, &SESSION];

/// Modes that tmux owns per pane. They never touch `client_key_table`, so the
/// badge reads `pane_mode` instead; that is also how the session tree stops
/// looking like copy mode.
pub struct PaneMode {
    pub name: &'static str,
    pub badge: &'static str,
    pub colour: &'static str,
    pub hints: &'static [Hint],
}

pub static TREE: PaneMode = PaneMode {
    name: "tree-mode",
    badge: "SESSIONS",
    colour: "#9fd3ff",
    hints: &[
        Hint {
            keys: "Enter",
            label: "open",
        },
        Hint {
            keys: "x",
            label: "kill",
        },
        Hint {
            keys: "q",
            label: "quit",
        },
    ],
};

pub static COPY: PaneMode = PaneMode {
    name: "copy-mode",
    badge: "COPY",
    colour: "#ffe08a",
    hints: &[
        Hint {
            keys: "drag",
            label: "select",
        },
        Hint {
            keys: "q",
            label: "quit",
        },
    ],
};

pub static PANE_MODES: &[&PaneMode] = &[&TREE, &COPY];

/// Direct actions available from every key table (zellij's Alt+ shortcuts).
pub static ALT: &[Binding] = &[
    bind(
        &["M-n"],
        hint("M-n", "new pane"),
        "split-window -h -c \"#{pane_current_path}\"",
        false,
    ),
    bind(
        &["M-h"],
        hint("M-hl", "focus"),
        "if-shell -F \"#{pane_at_left}\" \"previous-window\" \"select-pane -L\"",
        false,
    ),
    bind(
        &["M-l"],
        None,
        "if-shell -F \"#{pane_at_right}\" \"next-window\" \"select-pane -R\"",
        false,
    ),
    bind(
        &["M-j"],
        None,
        "if-shell -F \"#{pane_at_bottom}\" \"next-window\" \"select-pane -D\"",
        false,
    ),
    bind(
        &["M-k"],
        None,
        "if-shell -F \"#{pane_at_top}\" \"previous-window\" \"select-pane -U\"",
        false,
    ),
    bind(&["M-="], hint("M-=", "wide"), "resize-pane -R 3", false),
    bind(&["M--"], hint("M--", "narrow"), "resize-pane -L 3", false),
    bind(
        &["M-i"],
        hint("M-io", "move tab"),
        "swap-window -d -t -1",
        false,
    ),
    bind(&["M-o"], None, "swap-window -d -t +1", false),
];

/// Mouse drag always selects in tmux, even over a pane that grabbed the mouse
/// (vim, less, htop). tmux's own rule hands the drag to such a pane instead.
pub const MOUSE_DRAG: &str = "if -F \"#{pane_in_mode}\" \"send-keys -M\" \"copy-mode -M\"";

/// Bar colours.
pub const BAR_BG: &str = "#3c6f99";
pub const BAR_FG: &str = "#eaf2f8";
pub const LABEL_FG: &str = "#cfe0ee";
