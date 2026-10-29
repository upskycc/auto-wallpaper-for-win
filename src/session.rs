use std::path::Path;

#[cfg(windows)]
extern "system" {
    fn ProcessIdToSessionId(dwProcessId: u32, pSessionId: *mut u32) -> i32;
}

#[cfg(windows)]
pub fn is_interactive_session() -> bool {
    use windows::Win32::System::Threading::GetCurrentProcessId;
    let mut session = 0u32;
    unsafe {
        if ProcessIdToSessionId(GetCurrentProcessId(), &mut session) != 0 {
            session != 0
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn is_interactive_session() -> bool {
    true
}

#[cfg(windows)]
pub fn spawn_apply(kind: &str, image: &Path) -> anyhow::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
    use windows::Win32::System::Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock};
    use windows::Win32::System::RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken};
    use windows::Win32::System::Threading::{
        CreateProcessAsUserW, WaitForSingleObject, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT,
        PROCESS_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW,
    };

    let session = unsafe { WTSGetActiveConsoleSessionId() };
    if session == 0xFFFFFFFF {
        anyhow::bail!("no active console session");
    }

    let mut token = HANDLE::default();
    unsafe {
        WTSQueryUserToken(session, &mut token)
            .map_err(|e| anyhow::anyhow!("WTSQueryUserToken: {e}"))?;
    }

    let exe = std::env::current_exe().map_err(|e| anyhow::anyhow!("current_exe: {e}"))?;
    let image_abs = std::fs::canonicalize(image)
        .map_err(|e| anyhow::anyhow!("canonicalize {}: {e}", image.display()))?;
    let image_disp = crate::apply::win_display_path(&image_abs);
    let cmdline = format!("\"{}\" apply {} \"{}\"", exe.display(), kind, image_disp);
    let mut cmd_wide: Vec<u16> = cmdline.encode_utf16().chain(std::iter::once(0)).collect();
    let exe_wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    let mut env_block: *mut std::ffi::c_void = std::ptr::null_mut();
    let env_ok = unsafe { CreateEnvironmentBlock(&mut env_block, token, false) };

    let mut desktop: Vec<u16> = "winsta0\\default"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.lpDesktop = PWSTR(desktop.as_mut_ptr());
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    let created = if env_ok.is_ok() {
        unsafe {
            CreateProcessAsUserW(
                token,
                PCWSTR(exe_wide.as_ptr()),
                PWSTR(cmd_wide.as_mut_ptr()),
                None,
                None,
                false,
                CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                Some(env_block as *const std::ffi::c_void),
                None,
                &si,
                &mut pi,
            )
        }
    } else {
        unsafe {
            CreateProcessAsUserW(
                token,
                PCWSTR(exe_wide.as_ptr()),
                PWSTR(cmd_wide.as_mut_ptr()),
                None,
                None,
                false,
                CREATE_NO_WINDOW,
                None,
                None,
                &si,
                &mut pi,
            )
        }
    };

    unsafe {
        if !env_block.is_null() {
            let _ = DestroyEnvironmentBlock(env_block);
        }
        let _ = CloseHandle(token);
    }

    created.map_err(|e| anyhow::anyhow!("CreateProcessAsUserW: {e}"))?;

    unsafe {
        let wait = WaitForSingleObject(pi.hProcess, 30_000);
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
        if wait != WAIT_OBJECT_0 {
            anyhow::bail!("apply helper did not finish in 30s");
        }
    }
    Ok(())
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn spawn_apply(_kind: &str, _image: &Path) -> anyhow::Result<()> {
    Ok(())
}
