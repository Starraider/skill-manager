use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Skill {
    pub name: String,
    pub folder_path: PathBuf,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstalledSkill {
    pub name: String,
    pub harness_names: Vec<String>,
    pub installed_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeploymentResult {
    pub copied_paths: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub errors: Vec<(PathBuf, String)>,
}

