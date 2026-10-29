#[cfg(windows)]
use std::sync::atomic::AtomicBool;
#[cfg(windows)]
use std::sync::Arc;

#[cfg(windows)]
use crate::config;
#[cfg(windows)]
use crate::paths;
#[cfg(windows)]
use crate::scheduler;

#[cfg(windows)]
mod win {
    use super::*;
    use std::ffi::OsString;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use anyhow::Context;
    use windows_service::define_windows_service;
    use windows_service::service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    const SERVICE_NAME: &str = "WallShift";
    const SERVICE_DISPLAY: &str = "WallShift Wallpaper Service";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    define_windows_service!(ffi_service_main, service_main);

    pub fn install() -> anyhow::Result<()> {
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        let exe = std::env::current_exe()?;
        let info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY),
            service_type: SERVICE_TYPE,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe,
            launch_arguments: vec![OsString::from("service")],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };
        let service = manager.create_service(&info, ServiceAccess::CHANGE_CONFIG)?;
        service.set_description("Downloads HTTP images and rotates wallpaper and lock screen")?;
        println!("installed service {SERVICE_NAME}");
        Ok(())
    }

    pub fn uninstall() -> anyhow::Result<()> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
        )?;
        if let Ok(status) = service.query_status() {
            if status.current_state != ServiceState::Stopped {
                let _ = service.stop();
                std::thread::sleep(Duration::from_secs(2));
            }
        }
        service.delete()?;
        println!("removed service {SERVICE_NAME}");
        Ok(())
    }

    pub fn dispatch() -> anyhow::Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main).context("service dispatcher")
    }

    fn service_main(_args: Vec<OsString>) {
        let _ = run_service();
    }

    fn run_service() -> anyhow::Result<()> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_handler = stop.clone();
        let event_handler = move |control| match control {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                stop_for_handler.store(true, Ordering::SeqCst);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        };
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })?;

        let loaded = config::load_or_create(&paths::default_config_path())?;
        let result = scheduler::run_loop(loaded, stop, false);

        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;
        result
    }
}

#[cfg(windows)]
pub use win::{dispatch, install, uninstall};

#[cfg(not(windows))]
pub fn install() -> anyhow::Result<()> {
    anyhow::bail!("install is only available on Windows");
}

#[cfg(not(windows))]
pub fn uninstall() -> anyhow::Result<()> {
    anyhow::bail!("uninstall is only available on Windows");
}

#[cfg(not(windows))]
pub fn dispatch() -> anyhow::Result<()> {
    anyhow::bail!("service dispatcher is only available on Windows");
}
