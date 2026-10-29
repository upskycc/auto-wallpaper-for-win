use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

use crate::paths;

const DEFAULT_SOURCE: &str = "https://wp.upx8.com/api.php?resolution=3840x2160";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Selection {
    Random,
    RoundRobin,
}

impl Default for Selection {
    fn default() -> Self {
        Self::Random
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCfg {
    #[serde(default = "paths::configured_cache_dir")]
    pub cache_dir: PathBuf,
}

impl Default for ServiceCfg {
    fn default() -> Self {
        Self {
            cache_dir: paths::configured_cache_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCfg {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub interval_minutes: u64,
    #[serde(default)]
    pub selection: Selection,
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    60
}

fn default_sources() -> Vec<String> {
    vec![DEFAULT_SOURCE.to_string()]
}

impl ChannelCfg {
    fn wallpaper_default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 60,
            selection: Selection::Random,
            sources: default_sources(),
        }
    }

    fn lockscreen_default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 120,
            selection: Selection::Random,
            sources: default_sources(),
        }
    }

    pub fn validate(&self, name: &str) -> anyhow::Result<()> {
        if self.interval_minutes < 1 {
            bail!("{name}.interval_minutes must be >= 1");
        }
        if self.enabled && self.sources.is_empty() {
            bail!("{name}.sources must contain at least one URL when enabled");
        }
        for url in &self.sources {
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                bail!("{name} source is not http(s): {url}");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub service: ServiceCfg,
    #[serde(default = "ChannelCfg::wallpaper_default")]
    pub wallpaper: ChannelCfg,
    #[serde(default = "ChannelCfg::lockscreen_default")]
    pub lockscreen: ChannelCfg,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service: ServiceCfg::default(),
            wallpaper: ChannelCfg::wallpaper_default(),
            lockscreen: ChannelCfg::lockscreen_default(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.wallpaper.validate("wallpaper")?;
        self.lockscreen.validate("lockscreen")?;
        Ok(())
    }

    pub fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: Config,
    pub path: PathBuf,
    mtime: Option<SystemTime>,
}

impl LoadedConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let config: Config = toml::from_str(&raw)
            .with_context(|| format!("parse config {}", path.display()))?;
        config.validate()?;
        let mtime = fs::metadata(path).ok().and_then(|m| m.modified().ok());
        Ok(Self {
            config,
            path: path.to_path_buf(),
            mtime,
        })
    }

    pub fn reload_if_changed(&mut self) -> anyhow::Result<bool> {
        let mtime = fs::metadata(&self.path).ok().and_then(|m| m.modified().ok());
        if mtime == self.mtime {
            return Ok(false);
        }
        match Self::load(&self.path) {
            Ok(next) => {
                *self = next;
                Ok(true)
            }
            Err(_) => {
                self.mtime = mtime;
                Ok(false)
            }
        }
    }
}

pub fn write_default(path: &Path) -> anyhow::Result<Config> {
    let config = Config::default();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, config.to_toml()?)?;
    Ok(config)
}

pub fn load_or_create(path: &Path) -> anyhow::Result<LoadedConfig> {
    if !path.exists() {
        write_default(path)?;
    }
    LoadedConfig::load(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_example_shape() {
        let text = r#"
[wallpaper]
enabled = true
interval_minutes = 15
selection = "round-robin"
sources = ["https://wp.upx8.com/api.php?resolution=3840x2160"]

[lockscreen]
enabled = false
interval_minutes = 30
sources = ["https://example.com/a.jpg", "https://example.com/b.jpg"]
"#;
        let cfg: Config = toml::from_str(text).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.wallpaper.interval_minutes, 15);
        assert_eq!(cfg.wallpaper.selection, Selection::RoundRobin);
        assert!(!cfg.lockscreen.enabled);
        assert_eq!(cfg.lockscreen.sources.len(), 2);
    }

    #[test]
    fn rejects_interval_zero() {
        let mut cfg = Config::default();
        cfg.wallpaper.interval_minutes = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_ftp_source() {
        let mut cfg = Config::default();
        cfg.wallpaper.sources = vec!["ftp://x/a.jpg".into()];
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn reload_keeps_old_on_invalid() {
        let dir = std::env::temp_dir().join(format!("wallshift-cfg-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        let mut loaded = LoadedConfig {
            config: Config::default(),
            path: path.clone(),
            mtime: None,
        };
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "not toml").unwrap();
        drop(f);
        let changed = loaded.reload_if_changed().unwrap();
        assert!(!changed);
        assert!(loaded.config.wallpaper.enabled);
    }
}
