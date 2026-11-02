use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::AppError;
use crate::models::config::{HarnessConfig, TargetConfig};
use crate::models::skill::{DeploymentResult, Skill};
use crate::services::config_store::expand_tilde;
use crate::services::fs_scanner::is_valid_skill_name;

/// Lexically normalizes a path by removing redundant `.` and resolving `..` where possible,
/// without querying the filesystem.
pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }
    components.into_iter().collect()
}

/// Recursively copies a skill directory to `dest_dir`, skipping directory symlinks
/// and never following symlinks.
fn copy_skill_directory(src_dir: &Path, dest_dir: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dest_dir)?;

    for entry_res in walkdir::WalkDir::new(src_dir).follow_links(false) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                return Err(e
                    .into_io_error()
                    .unwrap_or_else(|| std::io::Error::other("WalkDir error")));
            }
        };

        // Skip any symlinks to avoid directory loops and sandbox escapes
        if entry.path_is_symlink() {
            continue;
        }

        let rel = match entry.path().strip_prefix(src_dir) {
            Ok(r) => r,
            Err(e) => return Err(std::io::Error::other(e.to_string())),
        };

        if rel.as_os_str().is_empty() {
            continue;
        }

        let target_child = dest_dir.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_child)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target_child.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::copy(entry.path(), &target_child)?;
        }
    }

    Ok(())
}

/// Deploys a collection of skills across multiple target environments and configured harnesses.
/// Performs pre-flight destination resolution and lexical deduplication before copying.
pub fn deploy_skills(
    skills: &[Skill],
    targets: &[TargetConfig],
    harnesses: &[HarnessConfig],
) -> DeploymentResult {
    let mut result = DeploymentResult::default();

    // Map: destination_path -> source_skill
    // Using HashMap to deduplicate identical destination paths
    let mut dest_map: HashMap<PathBuf, &Skill> = HashMap::new();

    for target in targets {
        if target.is_global {
            for harness in harnesses {
                let global_path_ref = match harness.global_path.as_ref() {
                    Some(gp) if !gp.as_os_str().is_empty() => gp,
                    _ => {
                        result.warnings.push(format!(
                            "Harness '{}' has no global path configured.",
                            harness.name
                        ));
                        continue;
                    }
                };

                let global_dir = normalize_lexically(&expand_tilde(global_path_ref));

                for skill in skills {
                    if !is_valid_skill_name(&skill.name) {
                        result.errors.push((
                            PathBuf::from(&skill.name),
                            format!("Invalid skill name: '{}'", skill.name),
                        ));
                        continue;
                    }

                    let dest = normalize_lexically(&global_dir.join(&skill.name));
                    if !dest.starts_with(&global_dir) {
                        result.errors.push((
                            dest,
                            format!(
                                "Path traversal security violation: destination escapes global directory {:?}",
                                global_dir
                            ),
                        ));
                        continue;
                    }

                    dest_map.entry(dest).or_insert(skill);
                }
            }
        } else {
            let base_dir = normalize_lexically(&expand_tilde(&target.path));

            for harness in harnesses {
                if !harness.relative_path.is_relative()
                    || harness
                        .relative_path
                        .components()
                        .any(|c| matches!(c, Component::ParentDir))
                {
                    result.warnings.push(format!(
                        "Harness '{}' has an invalid relative path: {:?}",
                        harness.name, harness.relative_path
                    ));
                    continue;
                }

                let harness_dir = normalize_lexically(&base_dir.join(&harness.relative_path));

                for skill in skills {
                    if !is_valid_skill_name(&skill.name) {
                        result.errors.push((
                            PathBuf::from(&skill.name),
                            format!("Invalid skill name: '{}'", skill.name),
                        ));
                        continue;
                    }

                    let dest = normalize_lexically(&harness_dir.join(&skill.name));
                    if !dest.starts_with(&base_dir) {
                        result.errors.push((
                            dest,
                            format!(
                                "Path traversal security violation: destination escapes target directory {:?}",
                                base_dir
                            ),
                        ));
                        continue;
                    }

                    dest_map.entry(dest).or_insert(skill);
                }
            }
        }
    }

    // Execute deduplicated copies
    // Sort destinations for deterministic execution order
    let mut sorted_destinations: Vec<(PathBuf, &Skill)> = dest_map.into_iter().collect();
    sorted_destinations.sort_by(|a, b| a.0.cmp(&b.0));

    for (dest, skill) in sorted_destinations {
        match copy_skill_directory(&skill.folder_path, &dest) {
            Ok(()) => {
                result.copied_paths.push(dest);
            }
            Err(e) => {
                result.errors.push((dest, e.to_string()));
            }
        }
    }

    result
}

/// Deletes a deployed skill across all configured harness directories within the given target.
/// Validates deletion boundary containment and deduplicates removal paths.
pub fn remove_skill_from_target(
    skill_name: &str,
    target: &TargetConfig,
    harnesses: &[HarnessConfig],
) -> Result<(), AppError> {
    if !is_valid_skill_name(skill_name) {
        return Err(AppError::TraversalAttempt(format!(
            "Invalid skill name for removal: '{}'",
            skill_name
        )));
    }

    let mut paths_to_remove: HashSet<PathBuf> = HashSet::new();

    if target.is_global {
        for harness in harnesses {
            if let Some(ref gp) = harness.global_path {
                let global_dir = normalize_lexically(&expand_tilde(gp));
                let skill_dest = normalize_lexically(&global_dir.join(skill_name));
                if !skill_dest.starts_with(&global_dir) {
                    return Err(AppError::TraversalAttempt(format!(
                        "Target path escapes base directory: {:?}",
                        skill_dest
                    )));
                }
                if skill_dest.exists() {
                    paths_to_remove.insert(skill_dest);
                }
            }
        }
    } else {
        let base_dir = normalize_lexically(&expand_tilde(&target.path));
        for harness in harnesses {
            if !harness.relative_path.is_relative()
                || harness
                    .relative_path
                    .components()
                    .any(|c| matches!(c, Component::ParentDir))
            {
                continue;
            }

            let skill_dest =
                normalize_lexically(&base_dir.join(&harness.relative_path).join(skill_name));
            if !skill_dest.starts_with(&base_dir) {
                return Err(AppError::TraversalAttempt(format!(
                    "Target path escapes base directory: {:?}",
                    skill_dest
                )));
            }
            if skill_dest.exists() {
                paths_to_remove.insert(skill_dest);
            }
        }
    }

    for path in paths_to_remove {
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        }
    }

    Ok(())
}
