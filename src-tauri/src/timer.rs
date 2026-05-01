use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Instant};

use crate::settings::Settings;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
    Manual,
    Idle,
    OutsideHours,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StateKind {
    Running,
    Warning,
    OnBreak,
    Paused,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerStatus {
    pub state: StateKind,
    pub pause_reason: Option<PauseReason>,
    pub seconds_until_warning: Option<i64>,
    pub seconds_until_break: Option<i64>,
    pub seconds_remaining_in_break: Option<i64>,
    pub postponed_count: u32,
}

impl Default for TimerStatus {
    fn default() -> Self {
        Self {
            state: StateKind::Running,
            pause_reason: None,
            seconds_until_warning: None,
            seconds_until_break: None,
            seconds_remaining_in_break: None,
            postponed_count: 0,
        }
    }
}

pub type SharedStatus = Arc<RwLock<TimerStatus>>;

#[derive(Debug)]
pub enum TimerCommand {
    Start,
    Pause(PauseReason),
    Resume,
    TakeBreakNow,
    SkipNextBreak,
    PostponeBreak,
    /// Fired when settings change so the timer re-evaluates its deadlines.
    SettingsUpdated(Box<Settings>),
}

// ── Internal state machine ───────────────────────────────────────────────────

enum State {
    Running {
        next_warning_at: Instant,
        next_break_at: Instant,
    },
    Warning {
        break_at: Instant,
        postponed_count: u32,
    },
    OnBreak {
        started_at: Instant,
        ends_at: Instant,
        postponed_count: u32,
    },
    Paused {
        reason: PauseReason,
        /// How far into the current interval we were when we paused.
        elapsed_secs: u64,
        postponed_count: u32,
    },
}

fn secs_from_now(instant: Instant) -> i64 {
    let now = Instant::now();
    if instant > now {
        (instant - now).as_secs() as i64
    } else {
        -((now - instant).as_secs() as i64)
    }
}

fn build_status(state: &State, settings: &Settings) -> TimerStatus {
    match state {
        State::Running {
            next_warning_at,
            next_break_at,
        } => TimerStatus {
            state: StateKind::Running,
            pause_reason: None,
            seconds_until_warning: Some(secs_from_now(*next_warning_at)),
            seconds_until_break: Some(secs_from_now(*next_break_at)),
            seconds_remaining_in_break: None,
            postponed_count: 0,
        },
        State::Warning {
            break_at,
            postponed_count,
        } => TimerStatus {
            state: StateKind::Warning,
            pause_reason: None,
            seconds_until_warning: Some(0),
            seconds_until_break: Some(secs_from_now(*break_at)),
            seconds_remaining_in_break: None,
            postponed_count: *postponed_count,
        },
        State::OnBreak {
            ends_at,
            postponed_count,
            ..
        } => TimerStatus {
            state: StateKind::OnBreak,
            pause_reason: None,
            seconds_until_warning: None,
            seconds_until_break: None,
            seconds_remaining_in_break: Some(secs_from_now(*ends_at).max(0)),
            postponed_count: *postponed_count,
        },
        State::Paused {
            reason,
            elapsed_secs,
            postponed_count,
        } => {
            let remaining_secs =
                (settings.timer.interval_minutes as i64 * 60) - (*elapsed_secs as i64);
            TimerStatus {
                state: StateKind::Paused,
                pause_reason: Some(*reason),
                seconds_until_warning: Some(remaining_secs),
                seconds_until_break: Some(
                    remaining_secs + settings.reminders.warning_seconds as i64,
                ),
                seconds_remaining_in_break: None,
                postponed_count: *postponed_count,
            }
        }
    }
}

fn earliest_deadline(state: &State) -> Instant {
    match state {
        State::Running {
            next_warning_at,
            next_break_at,
        } => (*next_warning_at).min(*next_break_at),
        State::Warning { break_at, .. } => *break_at,
        State::OnBreak { ends_at, .. } => *ends_at,
        // Paused: sleep for 1s so we can still emit ticks for status updates.
        State::Paused { .. } => Instant::now() + Duration::from_secs(1),
    }
}

// ── Timer task ───────────────────────────────────────────────────────────────

pub fn spawn(
    app: AppHandle,
    initial_settings: Settings,
    status: SharedStatus,
) -> mpsc::Sender<TimerCommand> {
    let (tx, mut rx) = mpsc::channel::<TimerCommand>(32);

    tokio::spawn(async move {
        let mut settings = initial_settings;
        let mut state = make_running_fresh(&settings);

        loop {
            let deadline = earliest_deadline(&state);

            // Update shared status for get_timer_status reads.
            {
                let s = build_status(&state, &settings);
                *status.write().unwrap() = s.clone();
                let _ = app.emit("timer:tick", s);
            }

            tokio::select! {
                _ = sleep_until(deadline) => {
                    state = on_deadline(state, &settings, &app, &status);
                }
                Some(cmd) = rx.recv() => {
                    state = on_command(state, cmd, &mut settings, &app, &status);
                }
            }
        }
    });

    tx
}

fn make_running_fresh(settings: &Settings) -> State {
    let interval = Duration::from_secs(settings.timer.interval_minutes as u64 * 60);
    let warning = Duration::from_secs(settings.reminders.warning_seconds as u64);
    let next_break_at = Instant::now() + interval;
    let next_warning_at = if warning.is_zero() {
        next_break_at
    } else {
        next_break_at - warning
    };
    State::Running {
        next_warning_at: next_warning_at.max(Instant::now()),
        next_break_at,
    }
}

fn on_deadline(state: State, settings: &Settings, app: &AppHandle, status: &SharedStatus) -> State {
    match state {
        State::Running {
            next_warning_at,
            next_break_at,
        } => {
            let now = Instant::now();
            if now >= next_break_at {
                // Skipped past warning (warning_seconds == 0), go straight to break.
                start_break(app, status, settings, 0)
            } else if now >= next_warning_at {
                let new_state = State::Warning {
                    break_at: next_break_at,
                    postponed_count: 0,
                };
                let s = build_status(&new_state, settings);
                *status.write().unwrap() = s.clone();
                let _ = app.emit("timer:warning_started", s);
                let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
                new_state
            } else {
                // Not yet — keep running (shouldn't normally happen).
                State::Running {
                    next_warning_at,
                    next_break_at,
                }
            }
        }
        State::Warning {
            break_at: _,
            postponed_count,
        } => start_break(app, status, settings, postponed_count),
        State::OnBreak {
            started_at: _,
            ends_at: _,
            postponed_count: _,
        } => {
            let _ = app.emit("timer:break_ended", ());
            let new_state = make_running_fresh(settings);
            let s = build_status(&new_state, settings);
            *status.write().unwrap() = s.clone();
            let _ = app.emit("timer:state_changed", s);
            new_state
        }
        State::Paused { .. } => {
            // Tick during paused — just stay paused (loop will sleep 1s again).
            state
        }
    }
}

fn start_break(
    app: &AppHandle,
    status: &SharedStatus,
    settings: &Settings,
    postponed_count: u32,
) -> State {
    let started_at = Instant::now();
    let ends_at = started_at + Duration::from_secs(settings.timer.break_seconds as u64);
    let new_state = State::OnBreak {
        started_at,
        ends_at,
        postponed_count,
    };
    let s = build_status(&new_state, settings);
    *status.write().unwrap() = s.clone();
    let _ = app.emit("timer:break_started", s);
    let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
    new_state
}

fn on_command(
    state: State,
    cmd: TimerCommand,
    settings: &mut Settings,
    app: &AppHandle,
    status: &SharedStatus,
) -> State {
    match cmd {
        TimerCommand::Start => {
            let new_state = make_running_fresh(settings);
            let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
            new_state
        }
        TimerCommand::Pause(reason) => {
            let elapsed_secs = match &state {
                State::Running { next_break_at, .. } => {
                    let total = settings.timer.interval_minutes as u64 * 60;
                    let remaining = secs_from_now(*next_break_at).max(0) as u64;
                    total.saturating_sub(remaining)
                }
                State::Paused { elapsed_secs, .. } => *elapsed_secs,
                _ => 0,
            };
            let postponed_count = match &state {
                State::Warning { postponed_count, .. } => *postponed_count,
                State::OnBreak { postponed_count, .. } => *postponed_count,
                _ => 0,
            };
            let new_state = State::Paused {
                reason,
                elapsed_secs,
                postponed_count,
            };
            let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
            new_state
        }
        TimerCommand::Resume => {
            // Reset to full interval on resume per spec (idle detection resets timer).
            let new_state = make_running_fresh(settings);
            let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
            new_state
        }
        TimerCommand::TakeBreakNow => start_break(app, status, settings, 0),
        TimerCommand::SkipNextBreak => {
            // Skip the next break: restart with a fresh full interval.
            let new_state = make_running_fresh(settings);
            let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
            new_state
        }
        TimerCommand::PostponeBreak => {
            let postponed_count = match &state {
                State::Warning { postponed_count, .. } => postponed_count + 1,
                State::OnBreak { postponed_count, .. } => postponed_count + 1,
                _ => 1,
            };
            let postpone_by = Duration::from_secs(5 * 60);
            let new_state = match state {
                State::Warning { break_at, .. } => State::Warning {
                    break_at: break_at + postpone_by,
                    postponed_count,
                },
                State::OnBreak { started_at, ends_at, .. } => State::OnBreak {
                    started_at,
                    ends_at: ends_at + postpone_by,
                    postponed_count,
                },
                other => {
                    // If not in warning/break, just reset the interval.
                    let _ = other;
                    make_running_fresh(settings)
                }
            };
            let _ = app.emit("timer:state_changed", build_status(&new_state, settings));
            new_state
        }
        TimerCommand::SettingsUpdated(new_settings) => {
            *settings = *new_settings;
            // If running, recompute deadline based on remaining time.
            match state {
                State::Running { next_break_at, .. } => {
                    let remaining =
                        Duration::from_secs(secs_from_now(next_break_at).max(0) as u64);
                    let warning =
                        Duration::from_secs(settings.reminders.warning_seconds as u64);
                    let next_warning_at = if warning >= remaining {
                        Instant::now()
                    } else {
                        Instant::now() + remaining - warning
                    };
                    State::Running {
                        next_warning_at,
                        next_break_at: Instant::now() + remaining,
                    }
                }
                other => other,
            }
        }
    }
}
