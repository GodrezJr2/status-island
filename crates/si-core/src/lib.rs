//! Status Island core: models, parsers, the live limit model and alerts.
//! Pure Rust with no OS dependencies, so everything here is unit-testable.

pub mod alerts;
pub mod claude;
pub mod codex;
pub mod estimate;
pub mod live;
pub mod model;
pub mod opencode;
pub mod scan;
pub mod tail;
