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
                // 1. OpenAI Codex
                // Native Standard: Uses .agents/skills for project and ~/.agents/skills globally.
                // Alternative proprietary/legacy: .codex/skills
                HarnessConfig {
                    id: "codex".to_string(),
                    name: "OpenAI Codex".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 2. Claude Code (Anthropic CLI)
                // Proprietary: Claude Code strictly scans .claude/ and ~/.claude/.
                // Does NOT natively discover .agents/skills.
                HarnessConfig {
                    id: "claude-code".to_string(),
                    name: "Claude Code".to_string(),
                    relative_path: PathBuf::from(".claude/skills"),
                    global_path: Some(PathBuf::from("~/.claude/skills")),
                    is_pinned: false,
                },
                // 3. Cursor (Cursor IDE)
                // Dual Support: Natively supports .agents/skills as well as .cursor/skills.
                // Preferred standard path selected to enable multi-agent sharing.
                // Alternative proprietary: .cursor/skills, ~/.cursor/skills
                HarnessConfig {
                    id: "cursor".to_string(),
                    name: "Cursor".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 4. Google Antigravity (Antigravity IDE & CLI)
                // Native Standard: Antigravity workspace customizations default to .agents/skills.
                // Alternative proprietary: .agent/skills, ~/.gemini/config/skills
                HarnessConfig {
                    id: "antigravity".to_string(),
                    name: "Google Antigravity".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 5. OpenCode (Anomaly Terminal Assistant)
                // Dual Support: Natively checks .opencode/skills, .agents/skills, and .claude/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .opencode/skills, ~/.config/opencode/skills
                HarnessConfig {
                    id: "opencode".to_string(),
                    name: "OpenCode".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 6. GitHub Copilot (VS Code Agent Mode & CLI)
                // Dual Support: Discovers .agents/skills alongside .github/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .github/skills, ~/.copilot/skills
                HarnessConfig {
                    id: "github-copilot".to_string(),
                    name: "GitHub Copilot".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 7. Windsurf (Codeium Cascade)
                // Dual Support: Cascade scans .windsurf/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .windsurf/skills, ~/.codeium/windsurf/skills
                HarnessConfig {
                    id: "windsurf".to_string(),
                    name: "Windsurf".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 8. Pi (pi-coding-agent)
                // Dual Support: Discovers .pi/skills and .agents/skills upon workspace trust.
                // Preferred standard path selected.
                // Alternative proprietary: .pi/skills, ~/.pi/agent/skills
                HarnessConfig {
                    id: "pi".to_string(),
                    name: "Pi".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 9. Cline (VS Code Extension)
                // Dual Support: Discovers .cline/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .cline/skills, ~/.cline/skills
                HarnessConfig {
                    id: "cline".to_string(),
                    name: "Cline".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
                // 10. Qoder (Qoder IDE & CLI)
                // Proprietary: Relies on .qoder/skills and ~/.qoder/skills for stable discovery.
                // Standard .agents/skills discovery is incomplete across releases.
                HarnessConfig {
                    id: "qoder".to_string(),
                    name: "Qoder".to_string(),
                    relative_path: PathBuf::from(".qoder/skills"),
                    global_path: Some(PathBuf::from("~/.qoder/skills")),
                    is_pinned: false,
                },
                // 11. Roo Code (VS Code Extension)
                // Dual Support: Discovers .roo/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .roo/skills, ~/.roo/skills
                HarnessConfig {
                    id: "roo-code".to_string(),
                    name: "Roo Code".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
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

