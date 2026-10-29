use std::path::Path;

#[cfg(windows)]
pub fn win_display_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
}

#[cfg(windows)]
fn wide_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
pub fn set_wallpaper(path: &Path) -> anyhow::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    };
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let abs = std::fs::canonicalize(path)
        .map_err(|e| anyhow::anyhow!("canonicalize {}: {e}", path.display()))?;
    let path_str = win_display_path(&abs);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let desk = hkcu
        .open_subkey_with_flags(r"Control Panel\Desktop", winreg::enums::KEY_WRITE)
        .map_err(|e| anyhow::anyhow!("open HKCU desktop key: {e}"))?;
    desk.set_value("Wallpaper", &path_str)
        .map_err(|e| anyhow::anyhow!("set Wallpaper: {e}"))?;
    desk.set_value("WallpaperStyle", &"10")
        .map_err(|e| anyhow::anyhow!("set WallpaperStyle: {e}"))?;
    desk.set_value("TileWallpaper", &"0")
        .map_err(|e| anyhow::anyhow!("set TileWallpaper: {e}"))?;

    let mut wide = wide_nul(&path_str);
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_mut_ptr().cast()),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
        .map_err(|e| anyhow::anyhow!("SPI_SETDESKWALLPAPER: {e}"))?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn set_wallpaper(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(windows)]
pub fn set_lock_screen(path: &Path) -> anyhow::Result<()> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let abs = std::fs::canonicalize(path)
        .map_err(|e| anyhow::anyhow!("canonicalize {}: {e}", path.display()))?;
    let dest_dir = crate::paths::default_cache_dir();
    std::fs::create_dir_all(&dest_dir)?;
    let ext = abs
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let dest = dest_dir.join(format!("lockscreen-applied.{ext}"));
    std::fs::copy(&abs, &dest)
        .map_err(|e| anyhow::anyhow!("copy lock image to {}: {e}", dest.display()))?;
    let dest_str = win_display_path(&dest);

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (csp, _) = hklm
        .create_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\PersonalizationCSP")
        .map_err(|e| anyhow::anyhow!("create PersonalizationCSP: {e}"))?;
    csp.set_value("LockScreenImagePath", &dest_str)
        .map_err(|e| anyhow::anyhow!("LockScreenImagePath: {e}"))?;
    csp.set_value("LockScreenImageUrl", &dest_str)
        .map_err(|e| anyhow::anyhow!("LockScreenImageUrl: {e}"))?;
    csp.set_value("LockScreenImageStatus", &1u32)
        .map_err(|e| anyhow::anyhow!("LockScreenImageStatus: {e}"))?;

    let (pol, _) = hklm
        .create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\Personalization")
        .map_err(|e| anyhow::anyhow!("create Personalization policy: {e}"))?;
    pol.set_value("LockScreenImage", &dest_str)
        .map_err(|e| anyhow::anyhow!("LockScreenImage: {e}"))?;

    Ok(())
}

#[cfg(not(windows))]
pub fn set_lock_screen(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

pub fn apply_wallpaper(path: &Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        if crate::session::is_interactive_session() {
            return set_wallpaper(path);
        }
        return crate::session::spawn_apply("wallpaper", path);
    }
    #[cfg(not(windows))]
    {
        set_wallpaper(path)
    }
}

pub fn apply_lock_screen(path: &Path) -> anyhow::Result<()> {
    set_lock_screen(path)
}
