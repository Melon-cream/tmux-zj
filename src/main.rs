//! tmux-zj: zellij-style modal keybindings for tmux.

mod model;
mod render;
mod tmux;

use std::process::ExitCode;

const USAGE: &str = "\
tmux-zj — zellij-style modal keybindings for tmux

Usage:
  tmux-zj init [options]     apply the keys and status bar to the running server
  tmux-zj config [options]   print the generated tmux config
  tmux-zj keys               print the keybinding cheat sheet
  tmux-zj help               show this text

Options:
  --no-status   leave the status bar alone (bindings only)
  --no-alt      do not bind the Alt+ direct actions
  --no-mouse    do not touch mouse or clipboard settings
  -L NAME       use this tmux socket name (tmux -L)
  -h, --help
  -V, --version

Add this to ~/.tmux.conf to load it on every start:
  run-shell \"tmux-zj init\"
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    };

    match command.as_str() {
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        "-V" | "--version" => {
            println!("tmux-zj {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        _ => {}
    }

    let mut options = render::Options {
        status: true,
        alt: true,
        mouse: true,
    };
    let mut socket: Option<String> = None;
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--no-status" => options.status = false,
            "--no-alt" => options.alt = false,
            "--no-mouse" => options.mouse = false,
            "-L" => match rest.next() {
                Some(name) => socket = Some(name.clone()),
                None => return fail("-L needs a socket name"),
            },
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return fail(&format!("unknown option '{other}'")),
        }
    }

    match command.as_str() {
        "init" => tmux::apply(&render::config(&options), socket.as_deref()),
        "config" => {
            print!("{}", render::config(&options));
            ExitCode::SUCCESS
        }
        "keys" => {
            print!("{}", render::keys());
            ExitCode::SUCCESS
        }
        other => fail(&format!("unknown command '{other}'")),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("tmux-zj: {message}");
    eprintln!("try 'tmux-zj help'");
    ExitCode::FAILURE
}
