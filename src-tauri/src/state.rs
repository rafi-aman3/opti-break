use std::path::PathBuf;
use std::sync::RwLock;

use crate::settings::Settings;

pub struct AppState {
    pub settings: RwLock<Settings>,
    pub settings_path: PathBuf,
}

impl AppState {
    pub fn new(settings: Settings, settings_path: PathBuf) -> Self {
        Self {
            settings: RwLock::new(settings),
            settings_path,
        }
    }
}
