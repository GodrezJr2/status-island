use status_island::config::{load_from, push_allowed, save_to, Config};

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("si-config-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.join("config.json")
}

#[test]
fn config_roundtrip_and_defaults() {
    let path = tmp("roundtrip");
    let defaults = load_from(&path);
    assert_eq!(defaults, Config::default());
    assert_eq!(defaults.monitor, "active");
    assert_eq!(defaults.ntfy.server, "https://ntfy.sh");
    assert!(!defaults.onboarded);
    assert!(defaults.quiet_hours.is_none());

    let mut cfg = Config::default();
    cfg.offset_x = -40;
    cfg.quiet_hours = Some([22, 7]);
    cfg.ntfy.topic = "si-test".into();
    cfg.prefs.task_finished = true;
    cfg.onboarded = true;
    save_to(&path, &cfg).unwrap();
    assert_eq!(load_from(&path), cfg);

    std::fs::write(&path, "{ not json").unwrap();
    assert_eq!(
        load_from(&path),
        Config::default(),
        "a corrupt file falls back to defaults"
    );
}

#[test]
fn quiet_hours_suppress_push() {
    let mut cfg = Config::default();
    cfg.ntfy.topic = "si-test".into();
    assert!(push_allowed(&cfg, 23, 0));

    cfg.quiet_hours = Some([22, 7]); // overnight window
    assert!(!push_allowed(&cfg, 23, 0));
    assert!(!push_allowed(&cfg, 6, 0));
    assert!(push_allowed(&cfg, 7, 0));
    assert!(push_allowed(&cfg, 12, 0));

    cfg.quiet_hours = Some([13, 14]); // same-day window
    assert!(!push_allowed(&cfg, 13, 0));
    assert!(push_allowed(&cfg, 14, 0));

    cfg.quiet_hours = None;
    cfg.alerts_paused_until_ms = Some(10_000);
    assert!(!push_allowed(&cfg, 12, 9_999));
    assert!(push_allowed(&cfg, 12, 10_000));

    cfg.alerts_paused_until_ms = None;
    cfg.ntfy.topic.clear();
    assert!(!push_allowed(&cfg, 12, 0), "no phone paired means no push");
}
