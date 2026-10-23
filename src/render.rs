//! Turns the model into tmux commands and format strings.

use crate::model::*;

pub struct Options {
    pub status: bool,
    pub alt: bool,
    pub mouse: bool,
}

const RESET: &str = "#[default]";
const SEP: &str = "  ";
const BADGE_FG: &str = "#10212b";
const PICK_BG: &str = "#7fb6dd";
const PICK_FG: &str = "#10212b";

// tmux splits `#{?cond,yes,no}` on every comma, including the ones inside a
// style directive, so styles used inside a branch must carry no commas at all.
fn keys_fg(colour: &str) -> String {
    format!("#[fg={}]#[bold]", colour)
}

fn badge_bg(colour: &str) -> String {
    format!("#[bg={}]#[fg={}]#[bold]", colour, BADGE_FG)
}

/// `#{?cond,if,else}` — no spaces around the commas, or they end up on screen.
fn branch(cond: &str, yes: &str, no: &str) -> String {
    format!("#{{?{},{},{}}}", cond, yes, no)
}

fn table_is(table: &str) -> String {
    format!("#{{==:#{{client_key_table}},{}}}", table)
}

fn pane_mode_is(name: &str) -> String {
    format!("#{{==:#{{pane_mode}},{}}}", name)
}

/// Copy mode wins over the key table, so a mode badge must step aside for it.
fn not_pane_mode(name: &str) -> String {
    format!("#{{!=:#{{pane_mode}},{}}}", name)
}

fn hints(list: &[Hint], colour: &str) -> String {
    let parts: Vec<String> = list
        .iter()
        .map(|h| {
            format!(
                "{}{}{} #[fg={}]{}",
                keys_fg(colour),
                h.keys,
                RESET,
                LABEL_FG,
                h.label
            )
        })
        .collect();
    parts.join(SEP)
}

fn text_width(s: &str) -> usize {
    s.chars().filter(|c| *c != '\n').count()
}

fn hints_width(list: &[Hint]) -> usize {
    let body: usize = list
        .iter()
        .map(|h| text_width(h.keys) + 1 + text_width(h.label))
        .sum();
    body + list.len().saturating_sub(1) * SEP.len()
}

fn mode_hints(m: &Mode) -> Vec<Hint> {
    m.bindings
        .iter()
        .filter_map(|b| b.hint.as_ref())
        .map(|h| Hint {
            keys: h.keys,
            label: h.label,
        })
        .collect()
}

fn mode_bar(m: &Mode) -> String {
    let mut out = hints(&mode_hints(m), m.colour);
    out.push_str(SEP);
    out.push_str(&format!(
        "{}Esc{} #[fg={}]back",
        keys_fg(m.colour),
        RESET,
        LABEL_FG
    ));
    out
}

fn normal_bar() -> String {
    let parts: Vec<String> = MODES
        .iter()
        .map(|m| {
            let key = m.enter.replace("C-", "^").to_lowercase();
            format!(
                "{}{}{} #[fg={}]{}",
                keys_fg(m.colour),
                key,
                RESET,
                LABEL_FG,
                m.menu
            )
        })
        .collect();
    parts.join(SEP)
}

/// Width of a format string once the `#[...]` directives are gone.
fn strip_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_style = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if in_style {
            if c == ']' {
                in_style = false;
            }
            continue;
        }
        if c == '#' && chars.peek() == Some(&'[') {
            chars.next();
            in_style = true;
            continue;
        }
        width += 1;
    }
    width
}

/// Every possible state gets its own badge; only the active one carries text.
/// The state lives in tmux (`client_key_table`, `pane_mode`), so the bar cannot
/// get stuck on a stale mode.
pub fn status_left() -> String {
    let mut s = String::new();
    for m in MODES {
        let cond = format!("#{{&&:{},{}}}", table_is(m.table), not_pane_mode(COPY.name));
        s.push_str(&badge_bg(m.colour));
        s.push_str(&branch(&cond, &format!(" {} ", m.name), ""));
        s.push_str(RESET);
    }
    for pm in PANE_MODES {
        s.push_str(&badge_bg(pm.colour));
        s.push_str(&branch(
            &pane_mode_is(pm.name),
            &format!(" {}", pm.badge),
            "",
        ));
        s.push_str(RESET);
    }
    s.push_str(&format!(" #[fg={}]{}#S ", LABEL_FG, RESET));
    s
}

pub fn status_left_width() -> usize {
    let widest = MODES
        .iter()
        .map(|m| m.name.len())
        .chain(PANE_MODES.iter().map(|p| p.badge.len()))
        .max()
        .unwrap_or(0);
    // padding on both sides of the badge, one gap, then room for the session name
    widest + 2 + 1 + 18
}

pub fn status_right() -> String {
    let mut s = normal_bar();
    for m in MODES.iter().rev() {
        s = branch(&table_is(m.table), &mode_bar(m), &s);
    }
    for pm in PANE_MODES.iter().rev() {
        s = branch(&pane_mode_is(pm.name), &hints(pm.hints, pm.colour), &s);
    }
    s
}

pub fn status_right_width() -> usize {
    let mut max = strip_width(&normal_bar());
    for m in MODES {
        let w = hints_width(&mode_hints(m)) + SEP.len() + 3 + 1 + 4;
        max = max.max(w);
    }
    for pm in PANE_MODES {
        max = max.max(hints_width(pm.hints));
    }
    max + 2
}

/// tmux drops back to the root table after handling a key, so every binding
/// has to name the table it wants to be in afterwards. A mode action that ends
/// the mode names `root` instead.
fn command(b: &Binding, table: &str) -> String {
    let cmd = b.cmd.to_string();
    if b.leave {
        format!("{} \\; switch-client -T {}", cmd, ROOT_TABLE)
    } else {
        // Re-enter the table first: if the action fails at an edge (no pane that
        // way, no previous window), tmux aborts the rest of the chain and the
        // mode would otherwise drop out from under the user.
        format!("switch-client -T {} \\; {}", table, cmd)
    }
}

pub fn config(o: &Options) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "# tmux-zj {} — zellij-style modal keys for tmux\n",
        env!("CARGO_PKG_VERSION")
    ));
    s.push_str("# Apply:  run-shell \"tmux-zj init\"    Inspect:  tmux-zj config\n\n");

    for m in MODES {
        s.push_str(&format!(
            "# --- {} mode: {} enters, Esc leaves\n",
            m.name, m.enter
        ));
        for b in m.bindings {
            for k in b.keys {
                s.push_str(&format!(
                    "bind-key -T {} {} {}\n",
                    m.table,
                    k,
                    command(b, m.table)
                ));
            }
        }
        for other in MODES.iter().filter(|o| o.table != m.table) {
            s.push_str(&format!(
                "bind-key -T {} {} switch-client -T {}\n",
                m.table, other.enter, other.table
            ));
        }
        s.push_str(&format!(
            "bind-key -T {} {} switch-client -T {}\n",
            m.table, m.enter, ROOT_TABLE
        ));
        for k in ["Escape", "Enter", "q"] {
            s.push_str(&format!(
                "bind-key -T {} {} switch-client -T {}\n",
                m.table, k, ROOT_TABLE
            ));
        }
        if o.alt {
            for b in ALT {
                for k in b.keys {
                    s.push_str(&format!(
                        "bind-key -T {} {} {}\n",
                        m.table,
                        k,
                        command(b, m.table)
                    ));
                }
            }
        }
        s.push('\n');
    }

    s.push_str("# --- enter modes from the root table\n");
    for m in MODES {
        s.push_str(&format!(
            "bind-key -n {} switch-client -T {}\n",
            m.enter, m.table
        ));
    }
    if o.alt {
        s.push('\n');
        s.push_str("# --- direct actions (zellij's Alt+ keys), from every table\n");
        for b in ALT {
            for k in b.keys {
                s.push_str(&format!("bind-key -n {} {}\n", k, b.cmd));
            }
        }
    }

    if o.mouse {
        s.push('\n');
        s.push_str("# --- mouse: drag selects and copies, even over a pane that\n");
        s.push_str("#     asked for the mouse itself (vim, less, htop)\n");
        s.push_str("set -g mouse on\n");
        s.push_str("set -g set-clipboard on\n");
        s.push_str(&format!("bind-key -n MouseDrag1Pane {}\n", MOUSE_DRAG));
    }

    if o.status {
        s.push('\n');
        s.push_str("# --- status bar driven by tmux's own state\n");
        s.push_str(&format!(
            "set -g status-style \"bg={},fg={}\"\n",
            BAR_BG, BAR_FG
        ));
        s.push_str(&format!(
            "set -g mode-style \"bg={},fg={}\"\n",
            PICK_BG, PICK_FG
        ));
        s.push_str(&format!(
            "set -g message-style \"bg={},fg={}\"\n",
            PICK_BG, PICK_FG
        ));
        s.push_str(&format!("set -g status-left \"{}\"\n", status_left()));
        s.push_str(&format!(
            "set -g status-left-length {}\n",
            status_left_width()
        ));
        s.push_str(&format!("set -g status-right \"{}\"\n", status_right()));
        s.push_str(&format!(
            "set -g status-right-length {}\n",
            status_right_width()
        ));
    }
    s
}

pub fn keys() -> String {
    let mut s = String::new();
    s.push_str("tmux-zj — zellij-style modal keys for tmux\n\n");
    s.push_str("Ctrl+<mode key> enters a mode. Esc, Enter or q leaves it.\n");
    s.push_str("Mode keys work inside every mode too, so you can hop straight across.\n\n");
    for m in MODES {
        s.push_str(&format!("{} ({})\n", m.name, m.enter));
        for h in mode_hints(m) {
            s.push_str(&format!("  {:<8} {}\n", h.keys, h.label));
        }
        s.push_str(&format!("  {:<8} {}\n\n", "Esc", "leave"));
    }
    s.push_str("Direct, no mode needed\n");
    for b in ALT {
        if let Some(h) = &b.hint {
            s.push_str(&format!("  {:<8} {}\n", h.keys, h.label));
        }
    }
    s.push_str("\nMouse\n");
    s.push_str("  drag     select and copy to the system clipboard\n");
    s.push_str("  wheel    scroll the pane\n");
    s
}
