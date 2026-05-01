use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;

use crate::db;
use crate::settings::Settings;
use crate::timer::{SharedStatus, TimerCommand, TimerStatus};

pub struct AppState {
    pub settings: Arc<RwLock<Settings>>,
    pub settings_path: PathBuf,
    pub timer_tx: mpsc::Sender<TimerCommand>,
    pub timer_status: SharedStatus,
    pub db: Option<db::DbHandle>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        settings_path: PathBuf,
        timer_tx: mpsc::Sender<TimerCommand>,
        timer_status: SharedStatus,
        db: Option<db::DbHandle>,
    ) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            settings_path,
            timer_tx,
            timer_status,
            db,
        }
    }

    pub fn settings_arc(&self) -> Arc<RwLock<Settings>> {
        Arc::clone(&self.settings)
    }

    pub fn status_snapshot(&self) -> TimerStatus {
        self.timer_status.read().unwrap().clone()
    }
}
