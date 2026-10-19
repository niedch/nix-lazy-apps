use crate::nix::model::{ActivityType, LogField, NixLogAction, NixLogMessage};
use std::collections::HashMap;

/// Snapshot of current progress passed to the UI or progress bar.
#[derive(Debug, Clone, PartialEq)]
pub struct NixProgressSnapshot {
    /// 0.0 to 1.0 if deterministic progress is known; None if indeterminate.
    pub fraction: Option<f32>,
    /// Summary for UI header (e.g., "Evaluating flake...", "Building derivation...").
    pub header: String,
    /// Detailed status for UI subtext (e.g., "copying mailcap.patch to the store").
    pub detail: String,
    /// Build counter: (done, expected, running)
    pub builds: (u64, u64, u64),
    /// Download/Copy counter: (done, expected, running)
    pub downloads: (u64, u64, u64),
    /// True when the root task finishes.
    pub is_finished: bool,
}

#[derive(Debug, Default)]
struct ProgressCounter {
    done: u64,
    expected: u64,
    running: u64,
    failed: u64,
}

#[derive(Debug)]
struct ActiveActivity {
    act_type: ActivityType,
    text: String,
}

#[derive(Debug, Default)]
pub struct NixStateTracker {
    activities: HashMap<u64, ActiveActivity>,
    build_progress: ProgressCounter,
    download_progress: ProgressCounter,
    last_detail: String,
    current_phase: Option<String>,
    root_activity_id: Option<u64>,
    is_finished: bool,
}

impl NixStateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a single line from stdout/stderr and update state.                                                                                                                                                                 
    pub fn process_line(&mut self, line: &str) -> NixProgressSnapshot {
        let msg = self.parse_line(line);
        self.apply_message(msg);
        self.snapshot()
    }

    fn parse_line(&self, line: &str) -> NixLogMessage {
        let trimmed = line.trim();
        if let Some(json_payload) = trimmed.strip_prefix("@nix ") {
            if let Ok(action) = serde_json::from_str::<NixLogAction>(json_payload) {
                return NixLogMessage::Event(action);
            }
        }
        NixLogMessage::Raw(trimmed.to_string())
    }

    fn apply_message(&mut self, msg: NixLogMessage) {
        match msg {
            NixLogMessage::Event(action) => match action {
                NixLogAction::Start {
                    id,
                    act_type,
                    text,
                    parent,
                    ..
                } => {
                    if parent == 0 && self.root_activity_id.is_none() {
                        self.root_activity_id = Some(id);
                    }
                    if !text.is_empty() {
                        self.last_detail = text.clone();
                    }
                    self.activities.insert(
                        id,
                        ActiveActivity {
                            act_type: ActivityType::from(act_type),
                            text,
                        },
                    );
                }
                NixLogAction::Stop { id } => {
                    self.activities.remove(&id);
                    if Some(id) == self.root_activity_id {
                        self.is_finished = true;
                    }
                }
                NixLogAction::Result {
                    id,
                    res_type,
                    fields,
                } => {
                    self.handle_result(id, res_type, &fields);
                }
                NixLogAction::SetPhase { phase } => {
                    self.current_phase = Some(phase);
                }
                NixLogAction::Msg { msg, .. } => {
                    self.last_detail = msg;
                }
            },
            NixLogMessage::Raw(raw_line) => {
                if !raw_line.is_empty() {
                    self.last_detail = raw_line;
                }
            }
        }
    }
}
