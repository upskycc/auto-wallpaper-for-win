use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use rand::Rng;

use crate::apply;
use crate::config::{ChannelCfg, LoadedConfig, Selection};
use crate::download;
use crate::paths;
use crate::state::State;

const TICK: Duration = Duration::from_secs(15);

#[derive(Clone, Copy)]
enum ChannelKind {
    Wallpaper,
    LockScreen,
}

impl ChannelKind {
    fn cache_file(self, cache_dir: &Path) -> PathBuf {
        match self {
            Self::Wallpaper => cache_dir.join("wallpaper.bin"),
            Self::LockScreen => cache_dir.join("lockscreen.bin"),
        }
    }
}

pub fn run_loop(mut loaded: LoadedConfig, stop: Arc<AtomicBool>, once: bool) -> anyhow::Result<()> {
    let cache = paths::resolve_cache_dir(&loaded.config.service.cache_dir);
    paths::ensure_dirs(&cache)?;
    let state_path = paths::state_path();
    let mut state = State::load(&state_path);

    loop {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let _ = loaded.reload_if_changed();
        let cfg = &loaded.config;
        let cache = paths::resolve_cache_dir(&cfg.service.cache_dir);
        paths::ensure_dirs(&cache)?;

        let now = unix_now();
        tick_channel(
            ChannelKind::Wallpaper,
            &cfg.wallpaper,
            &cache,
            now,
            &mut state.wallpaper_last_success,
            &mut state.wallpaper_index,
        );
        tick_channel(
            ChannelKind::LockScreen,
            &cfg.lockscreen,
            &cache,
            now,
            &mut state.lockscreen_last_success,
            &mut state.lockscreen_index,
        );
        let _ = state.save(&state_path);
        if once {
            break;
        }
        sleep_interruptible(TICK, &stop);
    }
    Ok(())
}

fn tick_channel(
    kind: ChannelKind,
    channel: &ChannelCfg,
    cache_dir: &Path,
    now: u64,
    last_success: &mut u64,
    index: &mut usize,
) {
    if !channel.enabled {
        return;
    }
    let due = *last_success == 0
        || now.saturating_sub(*last_success) >= channel.interval_minutes.saturating_mul(60);
    if !due {
        return;
    }
    if rotate(kind, channel, cache_dir, index).is_ok() {
        *last_success = now;
    }
}

fn rotate(
    kind: ChannelKind,
    channel: &ChannelCfg,
    cache_dir: &Path,
    index: &mut usize,
) -> anyhow::Result<()> {
    if channel.sources.is_empty() {
        anyhow::bail!("no sources");
    }
    let order = source_order(channel, index);
    let dest = kind.cache_file(cache_dir);
    let mut last_err = None;
    for url in order {
        match download::download_to(&url, &dest) {
            Ok(img) => {
                let named = dest.with_extension(img.ext());
                if named != dest {
                    std::fs::rename(&dest, &named)
                        .with_context(|| format!("rename {}", named.display()))?;
                }
                apply_kind(kind, &named)?;
                return Ok(());
            }
            Err(err) => {
                last_err = Some(err);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("all sources failed")))
}

fn source_order(channel: &ChannelCfg, index: &mut usize) -> Vec<String> {
    let n = channel.sources.len();
    match channel.selection {
        Selection::Random => {
            let mut urls = channel.sources.clone();
            let mut rng = rand::thread_rng();
            for i in (1..urls.len()).rev() {
                let j = rng.gen_range(0..=i);
                urls.swap(i, j);
            }
            urls
        }
        Selection::RoundRobin => {
            let start = *index % n;
            *index = start.wrapping_add(1);
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                out.push(channel.sources[(start + i) % n].clone());
            }
            out
        }
    }
}

fn apply_kind(kind: ChannelKind, path: &Path) -> anyhow::Result<()> {
    match kind {
        ChannelKind::Wallpaper => apply::apply_wallpaper(path).context("apply wallpaper"),
        ChannelKind::LockScreen => apply::apply_lock_screen(path).context("apply lock screen"),
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn sleep_interruptible(total: Duration, stop: &AtomicBool) {
    let slice = Duration::from_millis(250);
    let mut left = total;
    while left > Duration::ZERO {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let step = if left < slice { left } else { slice };
        thread::sleep(step);
        left -= step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_walks_list() {
        let channel = ChannelCfg {
            enabled: true,
            interval_minutes: 1,
            selection: Selection::RoundRobin,
            sources: vec!["https://a".into(), "https://b".into(), "https://c".into()],
        };
        let mut index = 0usize;
        let first = source_order(&channel, &mut index);
        let second = source_order(&channel, &mut index);
        assert_eq!(first[0], "https://a");
        assert_eq!(second[0], "https://b");
        assert_eq!(index, 2);
    }
}
