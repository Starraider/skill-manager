use crate::services::i18n::Language;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub left_pane_width: Option<f64>,
    pub sources: Vec<SourceConfig>,
    pub targets: Vec<TargetConfig>,
    pub harnesses: Vec<HarnessConfig>,
}

impl AppConfig {
    pub fn default_seeded() -> Self {
        Self {
            language: Language::English,
            left_pane_width: None,
            sources: Vec::new(),
            targets: vec![TargetConfig {
                id: "global".to_string(),
                name: "Global".to_string(),
                path: PathBuf::new(),
                is_global: true,
                is_pinned: true,
            }],
            harnesses: vec![
                HarnessConfig {
                    id: "codex".to_string(),
                    name: "Codex".to_string(),
                    relative_path: PathBuf::from(".codex/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                HarnessConfig {
                    id: "claude-code".to_string(),
                    name: "Claude Code".to_string(),
                    relative_path: PathBuf::from(".claude/skills"),
                    global_path: Some(PathBuf::from("~/.claude/skills")),
                    is_pinned: false,
                },
                HarnessConfig {
                    id: "cursor".to_string(),
                    name: "Cursor".to_string(),
                    relative_path: PathBuf::from(".cursor/skills"),
                    global_path: Some(PathBuf::from("~/.cursor/skills")),
                    is_pinned: false,
                },
                HarnessConfig {
                    id: "antigravity".to_string(),
                    name: "Antigravity".to_string(),
                    relative_path: PathBuf::from(".antigravity/skills"),
                    global_path: Some(PathBuf::from("~/.gemini/antigravity/skills")),
                    is_pinned: false,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SourceConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TargetConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub is_global: bool,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HarnessConfig {
    pub id: String,
    pub name: String,
    pub relative_path: PathBuf,
    pub global_path: Option<PathBuf>,
    pub is_pinned: bool,
}

