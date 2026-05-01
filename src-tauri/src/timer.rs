use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Instant};

use crate::db;
use crate::settings::Settings;
use crate::{shortcuts, windows};

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
    /// Pause with optional auto-resume after `duration` (None = indefinite).
    PauseFor(Option<Duration>),
    Resume,
    TakeBreakNow,
    SkipNextBreak,
    PostponeBreak,
    /// ESC pressed globally — skips warning or ends break early.
    EscPressed,
    /// Idle threshold crossed while Running — pause and reset on return.
    IdlePause,
    /// User activity detected after idle pause — restart fresh interval.
    IdleResume,
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
        elapsed_secs: u64,
        postponed_count: u32,
        auto_resume_at: Option<Instant>,
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

fn make_running_fresh(settings: &Settings) -> State {
    let interval = Duration::from_secs(settings.timer.interval_minutes as u64 * 60);
    let warning = Duration::from_secs(settings.reminders.warning_seconds as u64);
    let next_break_at = Instant::now() + interval;
    let next_warning_at = if warning.is_zero() || warning >= interval {
        next_break_at
    } else {
        next_break_at - warning
    };
    State::Running {
        next_warning_at,
        next_break_at,
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
            ..
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
        State::Paused {
            auto_resume_at: Some(t),
            ..
        } => *t,
        State::Paused {
            auto_resume_at: None,
            ..
        } => Instant::now() + Duration::from_secs(1),
    }
}

fn overlay_count(app: &AppHandle) -> u32 {
    app.webview_windows()
        .into_iter()
        .filter(|(l, _)| l.starts_with("overlay_"))
        .count() as u32
}

// ── Timer task ───────────────────────────────────────────────────────────────

pub fn spawn(
    app: AppHandle,
    initial_settings: Settings,
    status: SharedStatus,
    db: Option<db::DbHandle>,
) -> mpsc::Sender<TimerCommand> {
    let (tx, mut rx) = mpsc::channel::<TimerCommand>(32);
    let esc_tx = tx.clone();

    tokio::spawn(async move {
        let mut settings = initial_settings;
        let mut state = make_running_fresh(&settings);

        loop {
            {
                let s = build_status(&state, &settings);
                *status.write().unwrap() = s.clone();
                let _ = app.emit("timer:tick", &s);
            }

            tokio::select! {
                _ = sleep_until(earliest_deadline(&state)) => {
                    state = on_deadline(state, &settings, &app, &status, &esc_tx, &db);
                }
                Some(cmd) = rx.recv() => {
                    state = on_command(state, cmd, &mut settings, &app, &status, &esc_tx, &db);
                }
            }
        }
    });

    tx
}

fn on_deadline(
    state: State,
    settings: &Settings,
    app: &AppHandle,
    status: &SharedStatus,
    esc_tx: &mpsc::Sender<TimerCommand>,
    db: &Option<db::DbHandle>,
) -> State {
    match state {
        State::Running {
            next_warning_at,
            next_break_at,
        } => {
            let now = Instant::now();
            if now >= next_break_at {
                start_break(app, status, settings, 0, esc_tx)
            } else if now >= next_warning_at {
                let new_state = State::Warning {
                    break_at: next_break_at,
                    postponed_count: 0,
                };
                publish(app, status, &new_state, settings, "timer:warning_started");
                windows::show_warning(app).ok();
                shortcuts::register_esc(app, esc_tx.clone());
                new_state
            } else {
                State::Running {
                    next_warning_at,
                    next_break_at,
                }
            }
        }
        State::Warning { postponed_count, .. } => {
            windows::close_warning(app);
            start_break(app, status, settings, postponed_count, esc_tx)
        }
        State::OnBreak { started_at, .. } => {
            end_break(app, status, settings, started_at, false, db)
        }
        State::Paused {
            auto_resume_at: Some(_),
            ..
        } => {
            let new_state = make_running_fresh(settings);
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }
        State::Paused {
            auto_resume_at: None,
            ..
        } => state,
    }
}

fn start_break(
    app: &AppHandle,
    status: &SharedStatus,
    settings: &Settings,
    postponed_count: u32,
    esc_tx: &mpsc::Sender<TimerCommand>,
) -> State {
    let started_at = Instant::now();
    let ends_at = started_at + Duration::from_secs(settings.timer.break_seconds as u64);
    let new_state = State::OnBreak {
        started_at,
        ends_at,
        postponed_count,
    };
    publish(app, status, &new_state, settings, "timer:break_started");
    windows::show_overlay(app, settings).ok();
    shortcuts::register_esc(app, esc_tx.clone());
    new_state
}

fn end_break(
    app: &AppHandle,
    status: &SharedStatus,
    settings: &Settings,
    started_at: Instant,
    ended_early: bool,
    db: &Option<db::DbHandle>,
) -> State {
    shortcuts::unregister_esc(app);

    if let Some(handle) = db {
        let mc = overlay_count(app);
        if ended_early {
            let elapsed = (Instant::now() - started_at).as_secs() as u32;
            db::record_break_event(handle, "skipped", elapsed, mc);
        } else {
            db::record_break_event(handle, "completed", settings.timer.break_seconds, mc);
        }
    }

    let _ = app.emit("timer:break_ended", ());
    let app_clone = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        windows::close_overlay(&app_clone);
    });
    let new_state = make_running_fresh(settings);
    publish(app, status, &new_state, settings, "timer:state_changed");
    new_state
}

fn publish(
    app: &AppHandle,
    status: &SharedStatus,
    state: &State,
    settings: &Settings,
    event: &str,
) {
    let s = build_status(state, settings);
    *status.write().unwrap() = s.clone();
    let _ = app.emit(event, &s);
    if event != "timer:state_changed" {
        let _ = app.emit("timer:state_changed", &s);
    }
}

fn on_command(
    state: State,
    cmd: TimerCommand,
    settings: &mut Settings,
    app: &AppHandle,
    status: &SharedStatus,
    esc_tx: &mpsc::Sender<TimerCommand>,
    db: &Option<db::DbHandle>,
) -> State {
    match cmd {
        TimerCommand::Start => {
            let new_state = make_running_fresh(settings);
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }

        TimerCommand::EscPressed => match state {
            State::Warning { .. } => {
                windows::close_warning(app);
                shortcuts::unregister_esc(app);
                if let Some(handle) = db {
                    db::record_break_event(handle, "skipped", 0, 0);
                }
                let new_state = make_running_fresh(settings);
                publish(app, status, &new_state, settings, "timer:state_changed");
                new_state
            }
            State::OnBreak { started_at, .. } => {
                end_break(app, status, settings, started_at, true, db)
            }
            other => other,
        },

        TimerCommand::PauseFor(duration) => {
            // Extract started_at before consuming state via matches!
            let break_started_at = if let State::OnBreak { started_at, .. } = &state {
                Some(*started_at)
            } else {
                None
            };

            if matches!(state, State::Warning { .. }) {
                windows::close_warning(app);
                shortcuts::unregister_esc(app);
            }
            if let Some(started_at) = break_started_at {
                shortcuts::unregister_esc(app);
                if let Some(handle) = db {
                    let elapsed = (Instant::now() - started_at).as_secs() as u32;
                    db::record_break_event(handle, "skipped", elapsed, overlay_count(app));
                }
                let _ = app.emit("timer:break_ended", ());
                let app_clone = app.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    windows::close_overlay(&app_clone);
                });
            }
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
                State::Warning { postponed_count, .. }
                | State::OnBreak { postponed_count, .. } => *postponed_count,
                _ => 0,
            };
            let new_state = State::Paused {
                reason: PauseReason::Manual,
                elapsed_secs,
                postponed_count,
                auto_resume_at: duration.map(|d| Instant::now() + d),
            };
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }

        TimerCommand::Resume => {
            let new_state = make_running_fresh(settings);
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }

        TimerCommand::TakeBreakNow => {
            if matches!(state, State::Warning { .. }) {
                windows::close_warning(app);
            }
            start_break(app, status, settings, 0, esc_tx)
        }

        TimerCommand::SkipNextBreak => {
            if matches!(state, State::Warning { .. }) {
                windows::close_warning(app);
                shortcuts::unregister_esc(app);
                if let Some(handle) = db {
                    db::record_break_event(handle, "skipped", 0, 0);
                }
            }
            let new_state = make_running_fresh(settings);
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }

        TimerCommand::PostponeBreak => {
            if let Some(handle) = db {
                db::record_break_event(handle, "postponed", 0, 0);
            }
            let extra = Duration::from_secs(5 * 60);
            let new_state = match state {
                State::Warning {
                    break_at,
                    postponed_count,
                } => State::Warning {
                    break_at: break_at + extra,
                    postponed_count: postponed_count + 1,
                },
                State::OnBreak {
                    started_at,
                    ends_at,
                    postponed_count,
                } => State::OnBreak {
                    started_at,
                    ends_at: ends_at + extra,
                    postponed_count: postponed_count + 1,
                },
                other => {
                    let _ = other;
                    make_running_fresh(settings)
                }
            };
            publish(app, status, &new_state, settings, "timer:state_changed");
            new_state
        }

        TimerCommand::IdlePause => {
            if let State::Running { next_break_at, .. } = state {
                let total = settings.timer.interval_minutes as u64 * 60;
                let remaining = secs_from_now(next_break_at).max(0) as u64;
                let elapsed_secs = total.saturating_sub(remaining);
                let new_state = State::Paused {
                    reason: PauseReason::Idle,
                    elapsed_secs,
                    postponed_count: 0,
                    auto_resume_at: None,
                };
                publish(app, status, &new_state, settings, "timer:state_changed");
                new_state
            } else {
                state // Already paused or in warning/break — ignore
            }
        }

        TimerCommand::IdleResume => {
            if let State::Paused {
                reason: PauseReason::Idle,
                ..
            } = state
            {
                let new_state = make_running_fresh(settings);
                publish(app, status, &new_state, settings, "timer:state_changed");
                new_state
            } else {
                state // Not idle-paused — ignore
            }
        }

        TimerCommand::SettingsUpdated(new_settings) => {
            *settings = *new_settings;
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
