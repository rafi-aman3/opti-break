use chrono::{Datelike, Local, Timelike};

use crate::settings::Settings;

/// Returns `true` if the current local time falls within the configured active-hours
/// window, or if active hours are disabled / not configured.
pub fn is_within_active_hours(settings: &Settings) -> bool {
    if !settings.schedule.active_hours_enabled {
        return true;
    }
    let Some(ah) = &settings.schedule.active_hours else {
        return true;
    };
    if ah.days.is_empty() {
        return false;
    }

    let now = Local::now();
    // chrono: num_days_from_sunday → 0=Sun, 1=Mon … 6=Sat (matches our schema)
    let weekday = now.weekday().num_days_from_sunday() as u8;
    if !ah.days.contains(&weekday) {
        return false;
    }

    let current = now.hour() * 60 + now.minute();
    let start = parse_hhmm(&ah.start);
    let end = parse_hhmm(&ah.end);

    if end <= start {
        // Overnight window (e.g. 22:00–02:00)
        current >= start || current < end
    } else {
        current >= start && current < end
    }
}

fn parse_hhmm(hhmm: &str) -> u32 {
    let mut p = hhmm.splitn(2, ':');
    let h = p.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let m = p.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    h * 60 + m
}
