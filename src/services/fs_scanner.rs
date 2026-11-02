use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust2::YamlLoader;

use crate::error::AppError;
use crate::models::config::{HarnessConfig, TargetConfig};
use crate::models::skill::{InstalledSkill, Skill};
use crate::services::config_store::expand_tilde;

/// Validates whether a skill name contains only valid characters (alphanumeric, underscore, hyphen)
/// and no path separators or traversal tokens.
pub fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return false;
    }
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Valid regex");
    re.is_match(name)
}

/// Sanitizes a raw string into a safe skill name by keeping only ASCII alphanumeric, hyphen,
/// and underscore characters.
pub fn sanitize_skill_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if sanitized.is_empty() {
        "unnamed-skill".to_string()
    } else {
        sanitized
    }
}

/// Extracts YAML frontmatter from file content delimited by `---` lines.
fn extract_yaml_frontmatter(content: &str) -> Option<&str> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = &trimmed[3..];
    let newline_idx = rest.find('\n')?;
    let body = &rest[newline_idx + 1..];

    // Find the closing --- delimiter on a line of its own
    for (idx, _) in body.match_indices("---") {
        if idx == 0 || body.as_bytes()[idx - 1] == b'\n' {
            let after_dashes = &body[idx + 3..];
            let next_char = after_dashes.chars().next();
            if next_char.is_none()
                || next_char == Some('\n')
                || next_char == Some('\r')
                || after_dashes.starts_with(" \n")
                || after_dashes.starts_with(" \r\n")
            {
                return Some(body[..idx].trim_end_matches(['\r', '\n']));
            }
        }
    }
    None
}

/// Parses the frontmatter of a `SKILL.md` file, returning `(name, title, description)`.
pub fn parse_skill_frontmatter(content: &str) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(yaml_str) = extract_yaml_frontmatter(content) {
        if let Ok(docs) = YamlLoader::load_from_str(yaml_str) {
            if let Some(doc) = docs.first() {
                let name = doc["name"].as_str().map(|s| s.to_string());
                let title = doc["title"].as_str().map(|s| s.to_string());
                let description = doc["description"].as_str().map(|s| s.to_string());
                return (name, title, description);
            }
        }
    }
    (None, None, None)
}

/// Scans immediate child directories of `source_path` for valid agent skills marked with `SKILL.md`.
pub fn scan_skills(source_path: &Path) -> Result<Vec<Skill>, AppError> {
    let expanded_source = expand_tilde(source_path);
    if !expanded_source.exists() || !expanded_source.is_dir() {
        return Err(AppError::InvalidPath(expanded_source));
    }

    let mut skills = Vec::new();
    let entries = fs::read_dir(&expanded_source)?;

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        // Skip symlinks and non-directories
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }

        let child_dir = entry.path();
        let folder_basename = match child_dir.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        // Check for SKILL.md case-insensitively
        let mut marker_file = None;
        if let Ok(sub_entries) = fs::read_dir(&child_dir) {
            for sub_entry in sub_entries.flatten() {
                if let Ok(ft) = sub_entry.file_type() {
                    if !ft.is_dir() && !ft.is_symlink() {
                        let name = sub_entry.file_name();
                        if name.to_string_lossy().eq_ignore_ascii_case("skill.md") {
                            marker_file = Some(sub_entry.path());
                            break;
                        }
                    }
                }
            }
        }

        if let Some(marker_path) = marker_file {
            let content = fs::read_to_string(&marker_path).unwrap_or_default();
            let (raw_name, title, description) = parse_skill_frontmatter(&content);

            let skill_name = if let Some(n) = raw_name {
                if is_valid_skill_name(&n) {
                    n
                } else {
                    sanitize_skill_name(&folder_basename)
                }
            } else {
                sanitize_skill_name(&folder_basename)
            };

            skills.push(Skill {
                name: skill_name,
                folder_path: child_dir,
                title,
                description,
            });
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Scans a target for installed skills across all configured harnesses, grouping
/// matching skills by name with associated harness names and paths.
pub fn scan_installed_skills(
    target: &TargetConfig,
    harnesses: &[HarnessConfig],
) -> Vec<InstalledSkill> {
    use std::collections::BTreeMap;

    // Map: skill_name -> (HashSet<harness_name>, HashSet<PathBuf>)
    let mut skill_map: BTreeMap<String, (HashSet<String>, HashSet<PathBuf>)> = BTreeMap::new();

    for harness in harnesses {
        let harness_base_opt: Option<PathBuf> = if target.is_global {
            harness.global_path.as_ref().map(|p| expand_tilde(p))
        } else {
            let target_root = expand_tilde(&target.path);
            if harness.relative_path.is_relative()
                && !harness
                    .relative_path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                Some(target_root.join(&harness.relative_path))
            } else {
                None
            }
        };

        let harness_base = match harness_base_opt {
            Some(p) if p.exists() && p.is_dir() => p,
            _ => continue,
        };

        if let Ok(entries) = fs::read_dir(&harness_base) {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let ft = match entry.file_type() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if ft.is_symlink() || !ft.is_dir() {
                    continue;
                }

                let skill_dir = entry.path();
                let folder_name = match skill_dir.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };

                // Check for marker file inside installed skill
                let mut has_marker = false;
                if let Ok(sub_entries) = fs::read_dir(&skill_dir) {
                    for sub in sub_entries.flatten() {
                        if sub
                            .file_name()
                            .to_string_lossy()
                            .eq_ignore_ascii_case("skill.md")
                        {
                            has_marker = true;
                            break;
                        }
                    }
                }

                if has_marker {
                    let entry_record = skill_map
                        .entry(folder_name)
                        .or_insert_with(|| (HashSet::new(), HashSet::new()));
                    entry_record.0.insert(harness.name.clone());
                    entry_record.1.insert(skill_dir);
                }
            }
        }
    }

    let mut result: Vec<InstalledSkill> = skill_map
        .into_iter()
        .map(|(name, (harnesses, paths))| {
            let mut h_list: Vec<String> = harnesses.into_iter().collect();
            h_list.sort();
            let mut p_list: Vec<PathBuf> = paths.into_iter().collect();
            p_list.sort();
            InstalledSkill {
                name,
                harness_names: h_list,
                installed_paths: p_list,
            }
        })
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}
