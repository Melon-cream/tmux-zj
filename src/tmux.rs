//! Thin wrapper over the tmux client: version gate and config application.

use std::io::Write;
use std::process::{Command, ExitCode, Stdio};

const MIN_MINOR: u32 = 1; // 3.1: pane_at_* formats, #{==:}

pub fn apply(config: &str, socket: Option<&str>) -> ExitCode {
    match version(socket) {
        Ok(v) => {
            if !supported(&v) {
                eprintln!("tmux-zj: tmux {} is too old, 3.1 or newer is required", v);
                return ExitCode::FAILURE;
            }
        }
        Err(e) => {
            eprintln!("tmux-zj: {e}");
            return ExitCode::FAILURE;
        }
    }

    let mut cmd = base(socket);
    cmd.args(["source-file", "-"]);
    let mut child = match cmd.stdin(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tmux-zj: cannot run tmux: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(stdin) = child.stdin.as_mut() {
        if let Err(e) = stdin.write_all(config.as_bytes()) {
            eprintln!("tmux-zj: cannot write to tmux: {e}");
            return ExitCode::FAILURE;
        }
    }
    match child.wait_with_output() {
        Ok(out) if out.status.success() => ExitCode::SUCCESS,
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            let err = err.trim();
            if err.contains("no server running") || err.contains("error connecting to") {
                eprintln!("tmux-zj: no tmux server is running — start tmux, then try again");
            } else {
                eprintln!("tmux-zj: tmux rejected the config:\n{err}");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("tmux-zj: {e}");
            ExitCode::FAILURE
        }
    }
}

fn base(socket: Option<&str>) -> Command {
    let mut cmd = Command::new("tmux");
    if let Some(s) = socket {
        cmd.args(["-L", s]);
    }
    cmd
}

fn version(socket: Option<&str>) -> Result<String, String> {
    let out = base(socket)
        .arg("-V")
        .output()
        .map_err(|e| format!("cannot run tmux: {e}"))?;
    if !out.status.success() {
        return Err("cannot read the tmux version".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text.trim().trim_start_matches("tmux ").to_string())
}

fn supported(v: &str) -> bool {
    let mut parts = v.split('.');
    let major: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor: u32 = parts
        .next()
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    major > 3 || (major == 3 && minor >= MIN_MINOR)
}
