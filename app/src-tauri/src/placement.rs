#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub work_x: i32,
    pub work_y: i32,
    pub work_w: u32,
    pub work_h: u32,
    pub scale: f64,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonitorChoice {
    Active,
    Fixed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskbarInfo {
    pub monitor_id: String,
    pub top: i32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub monitor_id: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub scale: f64,
}

pub fn place(
    monitors: &[MonitorInfo],
    choice: MonitorChoice,
    offset_x: i32,
    taskbar: Option<TaskbarInfo>,
) -> Vec<Placement> {
    let selected = match choice {
        MonitorChoice::Active => monitors.iter().find(|m| m.active),
        MonitorChoice::Fixed(ref id) => monitors.iter().find(|m| &m.id == id),
    }
    .or_else(|| monitors.iter().find(|m| m.active))
    .or_else(|| monitors.first());
    let Some(m) = selected else {
        return Vec::new();
    };
    let scale = if m.scale.is_finite() && m.scale > 0.0 {
        m.scale
    } else {
        1.0
    };
    let w = (460.0 * scale).round().max(1.0) as u32;
    let h = (520.0 * scale).round().max(1.0) as u32;
    let w = w.min(m.work_w);
    let h = h.min(m.work_h);
    let wanted_x = m.x + (m.w as i32 - w as i32) / 2 + (offset_x as f64 * scale).round() as i32;
    let x = wanted_x.clamp(
        m.work_x,
        m.work_x
            .saturating_add(m.work_w as i32)
            .saturating_sub(w as i32),
    );
    let taskbar_bottom = taskbar
        .as_ref()
        .filter(|t| t.monitor_id == m.id && t.top <= m.y + 8)
        .map(|t| t.top.saturating_add(t.height as i32));
    let top = m.y.max(m.work_y).max(taskbar_bottom.unwrap_or(m.y));
    let wanted_y = top + (6.0 * scale).round() as i32;
    let y = wanted_y.clamp(
        m.work_y,
        m.work_y
            .saturating_add(m.work_h as i32)
            .saturating_sub(h as i32),
    );
    vec![Placement {
        monitor_id: m.id.clone(),
        x,
        y,
        w,
        h,
        scale,
    }]
}
