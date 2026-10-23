use crate::nix::model::{ActivityType, LogField, NixLogAction, NixLogMessage};
use crate::nix::util::ansi_escape::strip_ansi_escapes;
use crate::nix::util::package_name::extract_package_name;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Nix structured log result types (`res*` in Nix C++ source).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ResultType {
    FileTransfer = 100,
    CopyPath = 101,
    SetRound = 102,
    PostBuildLogLine = 103,
    SetPhase = 104,
    Progress = 105,
    SetExpected = 106,
    Other(u32),
}

impl From<u32> for ResultType {
    fn from(value: u32) -> Self {
        match value {
            100 => Self::FileTransfer,
            101 => Self::CopyPath,
            102 => Self::SetRound,
            103 => Self::PostBuildLogLine,
            104 => Self::SetPhase,
            105 => Self::Progress,
            106 => Self::SetExpected,
            other => Self::Other(other),
        }
    }
}

/// Global progress across all active Nix operations.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GlobalProgress {
    /// 0.0 to 1.0 progress fraction if deterministic progress is known; None if indeterminate.
    pub fraction: Option<f32>,
    /// Path substitution/fetching counter: (done, expected, running)
    pub paths: (u64, u64, u64),
    /// Derivation build counter: (done, expected, running)
    pub builds: (u64, u64, u64),
    /// Download byte counters: (downloaded_bytes, total_expected_bytes) if known.
    pub download_bytes: Option<(u64, u64)>,
}

impl GlobalProgress {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the progress fraction based on download bytes if available,
    /// otherwise falling back to paths and builds.
    pub fn calculate_fraction(&self) -> Option<f32> {
        if let Some((downloaded, total)) = self.download_bytes
            && total > 0
        {
            return Some((downloaded as f32 / total as f32).clamp(0.0, 1.0));
        }

        let total_expected = self.paths.1 + self.builds.1;
        let total_done = self.paths.0 + self.builds.0;
        if total_expected > 0 {
            Some((total_done as f32 / total_expected as f32).clamp(0.0, 1.0))
        } else {
            None
        }
    }
}

/// Progress of an individual ongoing activity (e.g., building a derivation or downloading a file).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IndividualProgress {
    /// Clean package or file name.
    pub name: String,
    /// 0.0 to 1.0 progress fraction if known for this activity.
    pub fraction: Option<f32>,
    /// (transferred_bytes, total_bytes) for downloads/copies.
    pub bytes: Option<(u64, u64)>,
    /// Build phase if applicable (e.g. "unpackPhase", "buildPhase").
    pub phase: Option<String>,
}

impl IndividualProgress {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fraction: None,
            bytes: None,
            phase: None,
        }
    }

    /// Sets the byte progress and updates the progress fraction accordingly.
    pub fn set_bytes(&mut self, current: u64, total: u64) {
        self.bytes = Some((current, total));
        if total > 0 {
            self.fraction = Some((current as f32 / total as f32).clamp(0.0, 1.0));
        }
    }

    /// Sets the current build phase.
    pub fn set_phase(&mut self, phase: impl Into<String>) {
        self.phase = Some(phase.into());
    }
}

/// Snapshot of current progress passed to the UI or progress bar.
#[derive(Debug, Clone, PartialEq)]
pub struct NixProgressSnapshot {
    /// Overall progress across all activities.
    pub global: GlobalProgress,
    /// Detailed progress of the active item (transfer or build).
    pub individual: Option<IndividualProgress>,
    /// Summary for UI header (e.g., "Downloading packages (1/3)...", "Complete").
    pub header: String,
    /// Detailed status for UI subtext.
    pub detail: String,
    /// Error message captured from failed actions.
    pub error: Option<String>,
    /// True when the root task finishes.
    pub is_finished: bool,

    // Backward compatibility fields:
    pub fraction: Option<f32>,
    pub builds: (u64, u64, u64),
    pub downloads: (u64, u64, u64),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ProgressCounter {
    done: u64,
    expected: u64,
    running: u64,
    failed: u64,
}

#[derive(Debug, Clone)]
struct ActiveActivity {
    act_type: ActivityType,
    parent: u64,
    name: Option<String>,
    text: String,
}

#[derive(Debug, Default)]
pub struct NixStateTracker {
    activities: HashMap<u64, ActiveActivity>,
    path_progress: ProgressCounter,
    build_progress: ProgressCounter,
    total_expected_download_bytes: u64,
    finished_download_bytes: u64,
    active_transfers: HashMap<u64, (String, u64, u64)>,
    active_build_phase: Option<String>,
    last_transfer_id: Option<u64>,
    peak_global_fraction: f32,
    root_activity_id: Option<u64>,
    is_finished: bool,
    last_detail: String,
    error_message: Option<String>,
}

impl NixStateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a single line from stdout/stderr, apply state updates, and return snapshot.
    pub fn process_line(&mut self, line: &str) -> NixProgressSnapshot {
        let msg = self.parse_line(line);
        self.apply_message(msg);
        self.snapshot()
    }

    fn parse_line(&self, line: &str) -> NixLogMessage {
        let trimmed = line.trim();
        if let Some(json_payload) = trimmed.strip_prefix("@nix ")
            && let Ok(action) = serde_json::from_str::<NixLogAction>(json_payload)
        {
            return NixLogMessage::Event(action);
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
                    fields,
                    ..
                } => {
                    self.handle_start(id, act_type, text, parent, &fields);
                }
                NixLogAction::Stop { id } => {
                    self.handle_stop(id);
                }
                NixLogAction::Result {
                    id,
                    res_type,
                    fields,
                } => {
                    self.handle_result(id, ResultType::from(res_type), &fields);
                }
                NixLogAction::SetPhase { phase } => {
                    self.active_build_phase = Some(phase);
                }
                NixLogAction::Msg { level, msg } => {
                    let clean = strip_ansi_escapes(&msg);
                    if level == 0 {
                        self.error_message = Some(clean.clone());
                    }
                    self.last_detail = clean;
                }
            },
            NixLogMessage::Raw(raw_line) => {
                let clean = strip_ansi_escapes(&raw_line);
                if clean.starts_with("error:") {
                    self.error_message = Some(clean.clone());
                }
                if !clean.is_empty() {
                    self.last_detail = clean;
                }
            }
        }
    }

    fn resolve_activity_name(
        &self,
        act_type: &ActivityType,
        text: &str,
        fields: &[LogField],
        parent: u64,
    ) -> Option<String> {
        match act_type {
            ActivityType::CopyPath | ActivityType::Build => {
                let candidate = fields
                    .first()
                    .and_then(|f| f.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(text);
                Some(extract_package_name(candidate))
            }
            ActivityType::FileTransfer => {
                if parent == 0 {
                    return None;
                }

                self.activities.get(&parent).and_then(|p| {
                    p.name.clone().or_else(|| {
                        if p.parent != 0 {
                            self.activities
                                .get(&p.parent)
                                .and_then(|gp| gp.name.clone())
                        } else {
                            None
                        }
                    })
                })
            }
            _ => None,
        }
    }

    fn handle_start(
        &mut self,
        id: u64,
        act_type: u32,
        text: String,
        parent: u64,
        fields: &[LogField],
    ) {
        let parsed_act_type = ActivityType::from(act_type);
        if parent == 0
            && parsed_act_type == ActivityType::Realise
            && self.root_activity_id.is_none()
        {
            self.root_activity_id = Some(id);
        }
        if !text.is_empty() {
            self.last_detail = text.clone();
        }

        let name = self.resolve_activity_name(&parsed_act_type, &text, fields, parent);

        if parsed_act_type == ActivityType::FileTransfer {
            let transfer_name = name.clone().unwrap_or_else(|| "download".to_string());
            self.active_transfers.insert(id, (transfer_name, 0, 0));
            self.last_transfer_id = Some(id);
        }

        self.activities.insert(
            id,
            ActiveActivity {
                act_type: parsed_act_type,
                parent,
                name,
                text,
            },
        );
    }

    fn handle_stop(&mut self, id: u64) {
        let stopped = self.activities.remove(&id);
        if let Some((_name, cur, total)) = self.active_transfers.remove(&id)
            && stopped
                .as_ref()
                .is_none_or(|a| a.act_type == ActivityType::FileTransfer)
        {
            let bytes_to_add = if total > 0 { total } else { cur };
            self.finished_download_bytes =
                self.finished_download_bytes.saturating_add(bytes_to_add);
        }
        if self.last_transfer_id == Some(id) {
            self.last_transfer_id = self.active_transfers.keys().next().copied();
        }
        if let Some(act) = &stopped
            && act.act_type == ActivityType::Build
        {
            self.active_build_phase = None;
        }
        if Some(id) == self.root_activity_id {
            self.is_finished = true;
            self.peak_global_fraction = 1.0;
            self.root_activity_id = None;
        }
    }

    fn handle_result(&mut self, id: u64, res_type: ResultType, fields: &[LogField]) {
        match res_type {
            ResultType::Progress => {
                let done = fields.first().and_then(|f| f.as_u64()).unwrap_or(0);
                let expected = fields.get(1).and_then(|f| f.as_u64()).unwrap_or(0);
                let running = fields.get(2).and_then(|f| f.as_u64()).unwrap_or(0);
                let failed = fields.get(3).and_then(|f| f.as_u64()).unwrap_or(0);

                if let Some(act) = self.activities.get(&id) {
                    match act.act_type {
                        ActivityType::CopyPaths => {
                            self.path_progress = ProgressCounter {
                                done,
                                expected,
                                running,
                                failed,
                            };
                        }
                        ActivityType::Builds => {
                            self.build_progress = ProgressCounter {
                                done,
                                expected,
                                running,
                                failed,
                            };
                        }
                        ActivityType::FileTransfer => {
                            let name = act
                                .name
                                .clone()
                                .filter(|s| !s.is_empty())
                                .or_else(|| {
                                    if !act.text.is_empty() {
                                        Some(extract_package_name(&act.text))
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_else(|| "download".to_string());
                            let old_total = self
                                .active_transfers
                                .get(&id)
                                .map(|(_, _, tot)| *tot)
                                .unwrap_or(0);
                            let total_bytes = if expected > 0 { expected } else { old_total };
                            self.active_transfers.insert(id, (name, done, total_bytes));
                            self.last_transfer_id = Some(id);
                        }
                        _ => {}
                    }
                }
            }
            ResultType::SetExpected => {
                let target_type = fields.first().and_then(|f| f.as_u64());
                let expected_bytes = fields.get(1).and_then(|f| f.as_u64()).unwrap_or(0);
                if target_type
                    .and_then(|t| u32::try_from(t).ok())
                    .map(ActivityType::from)
                    == Some(ActivityType::FileTransfer)
                {
                    self.total_expected_download_bytes = expected_bytes;
                }
            }
            ResultType::SetPhase => {
                if let Some(phase) = fields.first().and_then(|f| f.as_str()) {
                    self.active_build_phase = Some(phase.to_string());
                }
            }
            _ => {}
        }
    }

    pub fn snapshot(&mut self) -> NixProgressSnapshot {
        let global = self.calculate_global_progress();
        let individual = self.determine_individual_progress();
        let header = self.determine_header();

        NixProgressSnapshot {
            global: global.clone(),
            individual,
            header,
            detail: self.last_detail.clone(),
            error: self.error_message.clone(),
            is_finished: self.is_finished,
            fraction: global.fraction,
            builds: global.builds,
            downloads: global.paths,
        }
    }

    fn calculate_global_progress(&mut self) -> GlobalProgress {
        let current_active_bytes: u64 =
            self.active_transfers.values().map(|(_, cur, _)| *cur).sum();

        let download_bytes = if self.total_expected_download_bytes > 0 {
            let current_bytes = self
                .finished_download_bytes
                .saturating_add(current_active_bytes);
            Some((current_bytes, self.total_expected_download_bytes))
        } else {
            None
        };

        let mut global = GlobalProgress {
            fraction: None,
            paths: (
                self.path_progress.done,
                self.path_progress.expected,
                self.path_progress.running,
            ),
            builds: (
                self.build_progress.done,
                self.build_progress.expected,
                self.build_progress.running,
            ),
            download_bytes,
        };

        let fraction = if self.is_finished {
            self.peak_global_fraction = 1.0;
            Some(1.0)
        } else {
            global.calculate_fraction().map(|frac| {
                let mono = frac.max(self.peak_global_fraction);
                self.peak_global_fraction = mono;
                mono
            })
        };

        global.fraction = fraction;
        global
    }

    fn current_transfer_progress(&self) -> Option<IndividualProgress> {
        let (name, cur, total) = self
            .last_transfer_id
            .and_then(|id| self.active_transfers.get(&id))
            .or_else(|| {
                self.active_transfers
                    .values()
                    .max_by_key(|(_, cur, total)| (*total, *cur))
            })?;

        let mut ind = IndividualProgress::new(name.clone());
        ind.set_bytes(*cur, *total);
        Some(ind)
    }

    fn current_build_progress(&self) -> Option<IndividualProgress> {
        if self.build_progress.running == 0 {
            return None;
        }

        let build_name = self
            .activities
            .values()
            .find(|a| a.act_type == ActivityType::Build)
            .and_then(|a| a.name.as_deref())
            .unwrap_or("derivation");

        let mut ind = IndividualProgress::new(build_name);
        if let Some(phase) = &self.active_build_phase {
            ind.set_phase(phase.clone());
        }
        Some(ind)
    }

    fn determine_individual_progress(&self) -> Option<IndividualProgress> {
        self.current_transfer_progress()
            .or_else(|| self.current_build_progress())
    }

    fn determine_header(&self) -> String {
        if self.error_message.is_some() {
            return "Installation failed".to_string();
        }

        if self.is_finished {
            return "Complete".to_string();
        }

        if self.path_progress.running > 0 || !self.active_transfers.is_empty() {
            if self.path_progress.expected > 0 {
                return format!(
                    "Downloading packages ({}/{})...",
                    self.path_progress.done, self.path_progress.expected
                );
            }
            return "Downloading packages...".to_string();
        }

        if self.build_progress.running > 0 || self.build_progress.expected > 0 {
            let name = self
                .activities
                .values()
                .find(|a| a.act_type == ActivityType::Build)
                .and_then(|a| a.name.as_deref());

            return match (name, self.active_build_phase.as_deref()) {
                (Some(name), Some(phase)) => format!("Building {name} ({phase})..."),
                (Some(name), None) => format!("Building {name}..."),
                (None, Some(phase)) => format!("Building ({phase})..."),
                (None, None) if self.build_progress.expected > 0 => format!(
                    "Building derivations ({}/{})...",
                    self.build_progress.done, self.build_progress.expected
                ),
                (None, None) => "Building derivations...".to_string(),
            };
        }

        if self.path_progress.expected > 0 {
            return format!(
                "Fetching inputs ({}/{})...",
                self.path_progress.done, self.path_progress.expected
            );
        }

        "Evaluating Nix expression...".to_string()
    }
}
