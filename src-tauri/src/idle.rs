use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use user_idle::UserIdle;

use crate::settings::Settings;
use crate::timer::TimerCommand;

const POLL: Duration = Duration::from_secs(5);
/// Extra elapsed beyond POLL that indicates the machine likely slept.
const SLEEP_EXTRA: Duration = Duration::from_secs(30);

pub fn spawn(timer_tx: mpsc::Sender<TimerCommand>, settings: Arc<RwLock<Settings>>) {
    tokio::spawn(async move {
        let mut was_idle = false;
        let mut last_poll = Instant::now();

        loop {
            tokio::time::sleep(POLL).await;

            let now = Instant::now();
            let actual_elapsed = now.duration_since(last_poll);
            last_poll = now;

            // If actual elapsed >> POLL, the machine was likely asleep — treat as idle.
            if actual_elapsed > POLL + SLEEP_EXTRA {
                if !was_idle {
                    was_idle = true;
                    let _ = timer_tx.try_send(TimerCommand::IdlePause);
                }
                continue;
            }

            let threshold_secs = settings
                .read()
                .map(|s| s.schedule.idle_threshold_minutes as u64 * 60)
                .unwrap_or(180);

            let idle_secs = UserIdle::get_time()
                .map(|t| t.as_seconds())
                .unwrap_or(0);

            let is_idle = idle_secs >= threshold_secs;

            if is_idle && !was_idle {
                was_idle = true;
                let _ = timer_tx.try_send(TimerCommand::IdlePause);
            } else if !is_idle && was_idle {
                was_idle = false;
                let _ = timer_tx.try_send(TimerCommand::IdleResume);
            }
        }
    });
}
