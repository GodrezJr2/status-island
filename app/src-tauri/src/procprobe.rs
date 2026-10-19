//! Win32 process check for Claude session registry entries. One cheap syscall per
//! session per tick, instead of spawning `tasklist`.

use si_core::scan::ProcessProbe;
use windows::Win32::Foundation::{CloseHandle, FILETIME};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

const STILL_ACTIVE: u32 = 259;

pub struct Win32Probe;

impl ProcessProbe for Win32Probe {
    fn is_live(&self, pid: u32, proc_start: Option<&str>) -> bool {
        let Some(created) = creation_filetime(pid) else {
            return false;
        };
        match proc_start.and_then(|s| s.parse::<u64>().ok()) {
            Some(expected) => created == expected,
            None => true,
        }
    }
}

/// Creation time of a still-running process as a FILETIME value, the same unit as
/// Claude's `procStart`. `None` if it isn't running or can't be opened.
pub fn creation_filetime(pid: u32) -> Option<u64> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut code = 0u32;
        let running = GetExitCodeProcess(handle, &mut code).is_ok() && code == STILL_ACTIVE;
        let (mut created, mut exited, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        let timed =
            GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user).is_ok();
        let _ = CloseHandle(handle);
        (running && timed)
            .then(|| (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
    }
}
