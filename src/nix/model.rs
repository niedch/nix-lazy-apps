use serde::{Deserialize, Serialize};

// @nix {"action":"start","id":3143130081656832,"level":5,"parent":0,"text":"copying '/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-source/pkgs/development/compilers/go/iana-etc-1.25.patch' to the store","type":0}

/// Top-level message: either a parsed `@nix` event or a raw output line.
#[derive(Debug, Clone, PartialEq)]
pub enum NixLogMessage {
    Event(NixLogAction),
    Raw(String),
}

/// Strongly-typed event payload parsed from the JSON object.                                                                                                                                                                    
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum NixLogAction {
    Start {
        id: u64,
        #[serde(default)]
        level: u32,
        #[serde(default)]
        parent: u64,
        #[serde(rename = "type", default)]
        act_type: u32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        fields: Vec<LogField>,
    },
    Stop {
        id: u64,
    },
    Result {
        id: u64,
        #[serde(rename = "type")]
        res_type: u32,
        #[serde(default)]
        fields: Vec<LogField>,
    },
    SetPhase {
        phase: String,
    },
    Msg {
        #[serde(default)]
        level: u32,
        msg: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LogField {
    Number(u64),
    Text(String),
}

impl LogField {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Text(s) => s.parse().ok(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Number(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[repr(u32)]
pub enum ActionType {
    Unknown = 0,
    CopyPath = 100,
    FileTransfer = 101,
    Realise = 102,
    CopyPaths = 103,
    Builds = 104,
    Build = 105,
    OptimiseStore = 106,
    Substitute = 108,
    QueryPathInfo = 109,
    Other(u32),
}

impl From<u32> for ActionType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Unknown,
            100 => Self::CopyPath,
            101 => Self::FileTransfer,
            102 => Self::Realise,
            103 => Self::CopyPaths,
            104 => Self::Builds,
            105 => Self::Build,
            106 => Self::OptimiseStore,
            108 => Self::Substitute,
            109 => Self::QueryPathInfo,
            other => Self::Other(other),
        }
    }
}
