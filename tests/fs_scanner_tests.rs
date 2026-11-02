mod common;

use skill_manager::models::config::{HarnessConfig, TargetConfig};
use skill_manager::services::fs_scanner::{scan_installed_skills, scan_skills};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_scan_skills_discovers_valid_skill_marker() {
    let mock_source = common::create_mock_source_with_skills();
    let skills = scan_skills(mock_source.path()).expect("Scanning valid source should succeed");

    // We created git-helper, code-review, bad-yaml, and not-a-skill.
    // not-a-skill has no SKILL.md and should NOT be discovered.
    assert_eq!(skills.len(), 3);
    assert!(skills.iter().any(|s| s.name == "git-helper"));
}

#[test]
fn test_scan_skills_case_insensitive_marker() {
    let mock_source = common::create_mock_source_with_skills();
    let skills = scan_skills(mock_source.path()).expect("Scanning valid source should succeed");

    // code-review has lowercase skill.md
    let skill = skills.iter().find(|s| s.name == "code-review");
    assert!(skill.is_some(), "Lowercase skill.md should be discovered");
    let cr = skill.unwrap();
    assert_eq!(cr.title.as_deref(), Some("Code Reviewer"));
}

#[test]
fn test_scan_skills_parses_yaml_frontmatter() {
    let mock_source = common::create_mock_source_with_skills();
    let skills = scan_skills(mock_source.path()).expect("Scanning valid source should succeed");

    let git_helper = skills
        .iter()
        .find(|s| s.name == "git-helper")
        .expect("git-helper must be found");

    assert_eq!(git_helper.title.as_deref(), Some("Git Helper"));
    assert_eq!(
        git_helper.description.as_deref(),
        Some("Streamlines git workflows")
    );
}

#[test]
fn test_scan_skills_ignores_directory_symlinks() {
    let mock_source = common::create_mock_source_with_skills();
    let external_temp = tempfile::tempdir().expect("Create external dir");
    let ext_skill = external_temp.path().join("external-skill");
    fs::create_dir_all(&ext_skill).unwrap();
    fs::write(
        ext_skill.join("SKILL.md"),
        "---\nname: external-skill\n---\n",
    )
    .unwrap();

    // Create a symlink inside mock_source pointing to ext_skill
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = mock_source.path().join("symlinked-skill");
        let _ = symlink(&ext_skill, &link_path);
    }

    let skills = scan_skills(mock_source.path()).expect("Scan should succeed");
    assert!(
        !skills.iter().any(|s| s.name == "external-skill"),
        "Symlinked skill directory must be ignored"
    );
}

#[test]
fn test_scan_skills_malformed_frontmatter_fallback() {
    let mock_source = common::create_mock_source_with_skills();
    let skills = scan_skills(mock_source.path()).expect("Scan should succeed");

    // bad-yaml has unclosed bracket in frontmatter
    let bad_skill = skills
        .iter()
        .find(|s| s.name == "bad-yaml")
        .expect("bad-yaml should fall back to folder basename");

    assert_eq!(bad_skill.name, "bad-yaml");
    assert!(bad_skill.description.is_none() || bad_skill.description.as_deref() == Some(""));
}

#[test]
fn test_scan_installed_skills_aggregates_across_harnesses() {
    let mock_project = common::create_mock_project_target();
    let proj_path = mock_project.path();

    // Setup two harness directories containing the same skill name
    let codex_skill = proj_path.join(".codex/skills/shared-skill");
    fs::create_dir_all(&codex_skill).unwrap();
    fs::write(codex_skill.join("SKILL.md"), "# Shared Skill").unwrap();

    let claude_skill = proj_path.join(".claude/skills/shared-skill");
    fs::create_dir_all(&claude_skill).unwrap();
    fs::write(claude_skill.join("SKILL.md"), "# Shared Skill").unwrap();

    let target = TargetConfig {
        id: "proj".to_string(),
        name: "Test Project".to_string(),
        path: proj_path.to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    let harnesses = vec![
        HarnessConfig {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            relative_path: PathBuf::from(".codex/skills"),
            global_path: None,
            is_pinned: false,
        },
        HarnessConfig {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            relative_path: PathBuf::from(".claude/skills"),
            global_path: None,
            is_pinned: false,
        },
    ];

    let installed = scan_installed_skills(&target, &harnesses);
    assert_eq!(installed.len(), 1);
    let skill = &installed[0];
    assert_eq!(skill.name, "shared-skill");
    assert_eq!(skill.harness_names.len(), 2);
    assert!(skill.harness_names.contains(&"Codex".to_string()));
    assert!(skill.harness_names.contains(&"Claude Code".to_string()));
    assert_eq!(skill.installed_paths.len(), 2);
}
