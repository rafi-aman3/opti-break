use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;

use crate::settings::Settings;
use crate::timer::{SharedStatus, TimerCommand, TimerStatus};

pub struct AppState {
    pub settings: RwLock<Settings>,
    pub settings_path: PathBuf,
    pub timer_tx: mpsc::Sender<TimerCommand>,
    pub timer_status: SharedStatus,
}

impl AppState {
    pub fn new(
        settings: Settings,
        settings_path: PathBuf,
        timer_tx: mpsc::Sender<TimerCommand>,
        timer_status: SharedStatus,
    ) -> Self {
        Self {
            settings: RwLock::new(settings),
            settings_path,
            timer_tx,
            timer_status,
        }
    }

    pub fn status_snapshot(&self) -> TimerStatus {
        self.timer_status.read().unwrap().clone()
    }
}

pub type SharedTimerStatus = Arc<RwLock<TimerStatus>>;
