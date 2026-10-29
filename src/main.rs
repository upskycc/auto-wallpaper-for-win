mod apply;
mod config;
mod download;
mod paths;
mod scheduler;
mod service;
mod session;
mod state;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Context;

fn main() {
    if let Err(err) = real_main() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn real_main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let config_path = take_flag_value(&mut args, "--config")
        .map(PathBuf::from)
        .unwrap_or_else(paths::default_config_path);

    let cmd = args.first().map(String::as_str).unwrap_or("run");
    match cmd {
        "install" => service::install(),
        "uninstall" => service::uninstall(),
        "service" => service::dispatch(),
        "run" => {
            let loaded = config::load_or_create(&config_path)?;
            scheduler::run_loop(loaded, Arc::new(AtomicBool::new(false)), false)
        }
        "once" => {
            let loaded = config::load_or_create(&config_path)?;
            scheduler::run_loop(loaded, Arc::new(AtomicBool::new(false)), true)
        }
        "apply" => {
            let kind = args.get(1).map(String::as_str).unwrap_or("");
            let path = args.get(2).map(Path::new).context("apply <wallpaper|lockscreen> <path>")?;
            match kind {
                "wallpaper" => apply::set_wallpaper(path),
                "lockscreen" => apply::set_lock_screen(path),
                _ => anyhow::bail!("apply <wallpaper|lockscreen> <path>"),
            }
        }
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => anyhow::bail!("unknown command: {other}"),
    }
}

fn take_flag_value(args: &mut Vec<String>, flag: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == flag) {
        args.remove(pos);
        if pos < args.len() {
            return Some(args.remove(pos));
        }
    }
    None
}

fn print_help() {
    println!(
        "WallShift — rotate Windows wallpaper and lock screen from HTTP URLs

Usage:
  wallshift.exe install
  wallshift.exe uninstall
  wallshift.exe run [--config PATH]
  wallshift.exe once [--config PATH]
  wallshift.exe apply wallpaper <file>
  wallshift.exe apply lockscreen <file>

Config default: {}
",
        paths::default_config_path().display()
    );
}
