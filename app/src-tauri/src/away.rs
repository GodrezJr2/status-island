use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::{
    System::SystemInformation::GetTickCount,
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
    if LOCKED.load(Ordering::Relaxed) || SUSPENDING.load(Ordering::Relaxed) {
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
