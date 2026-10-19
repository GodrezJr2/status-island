use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::{
    System::{
        RemoteDesktop::{
            WTSFreeMemory, WTSQuerySessionInformationW, WTSSessionInfoEx, WTSINFOEXW,
            WTS_CURRENT_SESSION, WTS_SESSIONSTATE_LOCK,
        },
        SystemInformation::GetTickCount,
    },
    UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
};

static LOCKED: AtomicBool = AtomicBool::new(false);
static SUSPENDING: AtomicBool = AtomicBool::new(false);

pub fn set_locked(value: bool) {
    LOCKED.store(value, Ordering::Relaxed);
}
pub fn set_suspending(value: bool) {
    SUSPENDING.store(value, Ordering::Relaxed);
}

pub fn is_away() -> bool {
    if LOCKED.load(Ordering::Relaxed) || SUSPENDING.load(Ordering::Relaxed) || session_locked() {
        return true;
    }
    let mut input = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if !unsafe { GetLastInputInfo(&mut input) }.as_bool() {
        return false;
    }
    unsafe { GetTickCount() }.wrapping_sub(input.dwTime) >= 120_000
}

fn session_locked() -> bool {
    let mut buffer = windows::core::PWSTR::null();
    let mut bytes = 0u32;
    let result = unsafe {
        WTSQuerySessionInformationW(
            None,
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &mut buffer,
            &mut bytes,
        )
    };
    if result.is_err() || buffer.is_null() {
        return false;
    }
    let locked = if bytes as usize >= std::mem::size_of::<WTSINFOEXW>() {
        let info = unsafe { &*(buffer.0 as *const WTSINFOEXW) };
        info.Level == 1
            && unsafe { info.Data.WTSInfoExLevel1.SessionFlags } == WTS_SESSIONSTATE_LOCK as i32
    } else {
        false
    };
    unsafe {
        WTSFreeMemory(buffer.0.cast());
    }
    locked
}
