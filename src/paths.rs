use std::path::{Path, PathBuf};

pub fn install_root() -> PathBuf {
    if let Ok(p) = std::env::var("WALLSHIFT_HOME") {
        return PathBuf::from(p);
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if exe_dir.file_name().and_then(|n| n.to_str()) == Some("bin") {
        exe_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(exe_dir)
    } else {
        exe_dir
    }
}

pub fn default_config_path() -> PathBuf {
    install_root().join("config").join("config.toml")
}

#[allow(dead_code)]
pub fn default_cache_dir() -> PathBuf {
    install_root().join("cache")
}

pub fn configured_cache_dir() -> PathBuf {
    PathBuf::from("cache")
}

pub fn state_path() -> PathBuf {
    install_root().join("config").join("state.json")
}

pub fn resolve_cache_dir(configured: &Path) -> PathBuf {
    if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        install_root().join(configured)
    }
}

pub fn ensure_dirs(cache_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(install_root().join("config"))?;
    std::fs::create_dir_all(cache_dir)?;
    Ok(())
}
