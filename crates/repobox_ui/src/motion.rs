//! The motion language. Everything springs; nothing eases linearly.

use gpui_kit::SpringConfig;

/// Under-damped: overshoots, then wobbles to rest. Gel press and release.
pub fn jelly() -> SpringConfig {
    SpringConfig::new(420.0, 9.0, 1.0)
}

/// The leading edge of a squish knob: fast, barely overshoots.
pub fn lead() -> SpringConfig {
    SpringConfig::new(520.0, 30.0, 1.0)
}

/// The trailing edge: slower, so the knob stretches mid-flight.
pub fn trail() -> SpringConfig {
    SpringConfig::new(190.0, 19.0, 1.0)
}

/// Toggles, hovers, small lifts.
pub fn soft() -> SpringConfig {
    SpringConfig::new(300.0, 16.0, 1.0)
}
