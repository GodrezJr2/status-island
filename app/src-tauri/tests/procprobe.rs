use si_core::scan::ProcessProbe;
use status_island::procprobe::{creation_filetime, Win32Probe};

#[test]
fn live_process_with_matching_start_is_live() {
    let pid = std::process::id();
    let start = creation_filetime(pid)
        .expect("own creation time")
        .to_string();
    assert!(Win32Probe.is_live(pid, Some(&start)));
    assert!(Win32Probe.is_live(pid, None));
}

#[test]
fn reused_pid_with_other_start_is_not_live() {
    assert!(!Win32Probe.is_live(std::process::id(), Some("1")));
}

#[test]
fn missing_process_is_not_live() {
    assert!(!Win32Probe.is_live(0xFFFF_FFF0, None));
}
