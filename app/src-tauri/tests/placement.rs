use status_island::placement::{place, MonitorChoice, MonitorInfo, TaskbarInfo};

fn monitor(id: &str, x: i32, scale: f64, active: bool) -> MonitorInfo {
    MonitorInfo {
        id: id.into(),
        x,
        y: 0,
        w: 1920,
        h: 1080,
        work_x: x,
        work_y: 0,
        work_w: 1920,
        work_h: 1040,
        scale,
        active,
    }
}

#[test]
fn placement_centres_per_monitor() {
    let monitors = [monitor("a", 0, 1.0, true), monitor("b", 1920, 1.0, false)];
    let a = place(&monitors, MonitorChoice::Active, 0, None);
    let b = place(&monitors, MonitorChoice::Fixed("b".into()), 0, None);
    assert_eq!(
        (a[0].monitor_id.as_str(), a[0].x, a[0].y, a[0].w, a[0].h),
        ("a", 730, 6, 460, 520)
    );
    assert_eq!((b[0].monitor_id.as_str(), b[0].x), ("b", 2650));
}

#[test]
fn placement_handles_monitor_removal_and_scale() {
    let monitors = [monitor("a", 0, 1.25, true), monitor("b", 1920, 1.5, false)];
    let before = place(&monitors, MonitorChoice::Fixed("b".into()), 4000, None);
    assert_eq!(before[0].scale, 1.5);
    assert!(before[0].x >= 1920 && before[0].x + before[0].w as i32 <= 3840);
    let after = place(&monitors[..1], MonitorChoice::Fixed("b".into()), 4000, None);
    assert_eq!(after[0].monitor_id, "a");
    assert!(after[0].x >= 0 && after[0].x + after[0].w as i32 <= 1920);
}

#[test]
fn placement_respects_top_taskbar() {
    let monitors = [monitor("a", 0, 1.0, true)];
    let taskbar = TaskbarInfo {
        monitor_id: "a".into(),
        top: 0,
        height: 40,
    };
    assert_eq!(
        place(&monitors, MonitorChoice::Active, 0, Some(taskbar))[0].y,
        46
    );
}
