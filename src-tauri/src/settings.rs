use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettings {
    pub interval_minutes: u32,
    pub break_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakSettings {
    pub dim_opacity: f64,
    pub monitors: MonitorSelection,
    pub fade_in_ms: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorSelection {
    All,
    Primary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSettings {
    pub warning_seconds: u32,
    pub sound_enabled: bool,
    pub sound_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveHours {
    pub start: String,
    pub end: String,
    pub days: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSettings {
    pub active_hours_enabled: bool,
    pub active_hours: Option<ActiveHours>,
    pub idle_threshold_minutes: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub autostart: bool,
    pub theme: Theme,
    pub streaks_enabled: bool,
    /// Set to true after the user completes first-run onboarding.
    #[serde(default)]
    pub onboarded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub timer: TimerSettings,
    #[serde(rename = "break")]
    pub break_: BreakSettings,
    pub reminders: ReminderSettings,
    pub schedule: ScheduleSettings,
    pub general: GeneralSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timer: TimerSettings {
                interval_minutes: 20,
                break_seconds: 20,
            },
            break_: BreakSettings {
                dim_opacity: 0.78,
                monitors: MonitorSelection::All,
                fade_in_ms: 400,
            },
            reminders: ReminderSettings {
                warning_seconds: 10,
                sound_enabled: false,
                sound_id: "chime_soft".to_string(),
            },
            schedule: ScheduleSettings {
                active_hours_enabled: false,
                active_hours: None,
                idle_threshold_minutes: 3,
            },
            general: GeneralSettings {
                autostart: true,
                theme: Theme::System,
                streaks_enabled: true,
                onboarded: false,
            },
        }
    }
}

pub fn settings_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("settings.json")
}

pub fn load(path: &Path) -> Settings {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<Settings>(&text).unwrap_or_else(|err| {
            tracing::warn!("settings: parse failed ({err}); using defaults");
            Settings::default()
        }),
        Err(_) => Settings::default(),
    }
}

pub fn save(path: &Path, settings: &Settings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings).unwrap();
    fs::write(path, text)
}

/// Merge a JSON patch (any subset of the schema) into the current settings.
/// Returns the new settings if the merged result is valid; otherwise an error.
pub fn merge_patch(current: &Settings, patch: Value) -> Result<Settings, String> {
    let mut current_value =
        serde_json::to_value(current).map_err(|e| format!("serialize current: {e}"))?;
    deep_merge(&mut current_value, patch);
    serde_json::from_value::<Settings>(current_value).map_err(|e| format!("invalid settings: {e}"))
}

fn deep_merge(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(t), Value::Object(p)) => {
            for (k, v) in p {
                deep_merge(t.entry(k).or_insert(Value::Null), v);
            }
        }
        (slot, other) => *slot = other,
    }
}
