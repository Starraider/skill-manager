mod common;

use skill_manager::models::config::{HarnessConfig, TargetConfig};
use skill_manager::models::skill::Skill;
use skill_manager::services::deployer::{deploy_skills, remove_skill_from_target};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_deploy_creates_parent_directories() {
    let mock_source = common::create_mock_source_with_skills();
    let mock_target = common::create_mock_project_target();

    let skill = Skill {
        name: "git-helper".to_string(),
        folder_path: mock_source.path().join("git-helper"),
        title: Some("Git Helper".to_string()),
        description: None,
    };

    let target = TargetConfig {
        id: "target-1".to_string(),
        name: "App 1".to_string(),
        path: mock_target.path().to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    let harnesses = vec![HarnessConfig {
        id: "codex".to_string(),
        name: "Codex".to_string(),
        relative_path: PathBuf::from("deeply/nested/.codex/skills"),
        global_path: None,
        is_pinned: false,
    }];

    let res = deploy_skills(&[skill], &[target], &harnesses);
    assert_eq!(res.copied_paths.len(), 1);
    assert!(res.errors.is_empty());

    let deployed_file = mock_target
        .path()
        .join("deeply/nested/.codex/skills/git-helper/SKILL.md");
    assert!(
        deployed_file.exists(),
        "Deployed SKILL.md should exist in created parent path"
    );
}

#[test]
fn test_deploy_deduplicates_overlapping_destinations() {
    let mock_source = common::create_mock_source_with_skills();
    let mock_target = common::create_mock_project_target();

    let skill = Skill {
        name: "git-helper".to_string(),
        folder_path: mock_source.path().join("git-helper"),
        title: Some("Git Helper".to_string()),
        description: None,
    };

    let target = TargetConfig {
        id: "target-1".to_string(),
        name: "App 1".to_string(),
        path: mock_target.path().to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    // Two harnesses sharing the exact same relative destination
    let harnesses = vec![
        HarnessConfig {
            id: "harness-a".to_string(),
            name: "Harness A".to_string(),
            relative_path: PathBuf::from(".shared/skills"),
            global_path: None,
            is_pinned: false,
        },
        HarnessConfig {
            id: "harness-b".to_string(),
            name: "Harness B".to_string(),
            relative_path: PathBuf::from("./.shared/skills"),
            global_path: None,
            is_pinned: false,
        },
    ];

    let res = deploy_skills(&[skill], &[target], &harnesses);
    // Deduplication should ensure it was copied exactly once
    assert_eq!(res.copied_paths.len(), 1);
    assert!(res.errors.is_empty());
}

#[test]
fn test_deploy_enforces_destination_containment() {
    let mock_source = common::create_mock_source_with_skills();
    let mock_target = common::create_mock_project_target();

    let skill = Skill {
        name: "evil-skill".to_string(),
        folder_path: mock_source.path().join("git-helper"),
        title: None,
        description: None,
    };

    let target = TargetConfig {
        id: "target-1".to_string(),
        name: "App 1".to_string(),
        path: mock_target.path().to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    // Traversal in relative path
    let harnesses = vec![HarnessConfig {
        id: "harness-evil".to_string(),
        name: "Evil Harness".to_string(),
        relative_path: PathBuf::from("../../etc"),
        global_path: None,
        is_pinned: false,
    }];

    let res = deploy_skills(&[skill], &[target], &harnesses);
    assert!(res.copied_paths.is_empty());
    // Should warn or error on invalid relative path / traversal
    assert!(!res.warnings.is_empty() || !res.errors.is_empty());
}

#[test]
fn test_deploy_skips_internal_directory_symlinks() {
    let mock_source = common::create_mock_source_with_skills();
    let mock_target = common::create_mock_project_target();

    let skill_folder = mock_source.path().join("git-helper");

    // Create an external secret directory
    let secret_dir = tempfile::tempdir().expect("Create secret dir");
    fs::write(secret_dir.path().join("secret.txt"), "TOP SECRET").unwrap();

    // Create symlink inside skill_folder pointing to secret_dir
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = skill_folder.join("symlink_to_secret");
        let _ = symlink(secret_dir.path(), &link_path);
    }

    let skill = Skill {
        name: "git-helper".to_string(),
        folder_path: skill_folder,
        title: Some("Git Helper".to_string()),
        description: None,
    };

    let target = TargetConfig {
        id: "target-1".to_string(),
        name: "App 1".to_string(),
        path: mock_target.path().to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    let harnesses = vec![HarnessConfig {
        id: "codex".to_string(),
        name: "Codex".to_string(),
        relative_path: PathBuf::from(".codex/skills"),
        global_path: None,
        is_pinned: false,
    }];

    let res = deploy_skills(&[skill], &[target], &harnesses);
    assert_eq!(res.copied_paths.len(), 1);

    let copied_secret = mock_target
        .path()
        .join(".codex/skills/git-helper/symlink_to_secret/secret.txt");
    assert!(
        !copied_secret.exists(),
        "Symlinked directories must not be followed or copied"
    );
}

#[test]
fn test_deploy_reports_error_on_locked_destination() {
    let mock_source = common::create_mock_source_with_skills();
    let mock_target = common::create_mock_project_target();

    let skill = Skill {
        name: "git-helper".to_string(),
        folder_path: mock_source.path().join("git-helper"),
        title: Some("Git Helper".to_string()),
        description: None,
    };

    let target = TargetConfig {
        id: "target-1".to_string(),
        name: "App 1".to_string(),
        path: mock_target.path().to_path_buf(),
        is_global: false,
        is_pinned: false,
    };

    let harnesses = vec![HarnessConfig {
        id: "codex".to_string(),
        name: "Codex".to_string(),
        relative_path: PathBuf::from(".codex/skills"),
        global_path: None,
        is_pinned: false,
    }];

    // Create a read-only file where the destination directory should be
    let dest_conflict = mock_target.path().join(".codex/skills/git-helper");
    fs::create_dir_all(dest_conflict.parent().unwrap()).unwrap();
    fs::write(&dest_conflict, "blocker file").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_conflict).unwrap().permissions();
        perms.set_mode(0o444);
        let _ = fs::set_permissions(&dest_conflict, perms);
        // Also make parent read-only so removing/overwriting the blocker file fails
        let mut parent_perms = fs::metadata(dest_conflict.parent().unwrap())
            .unwrap()
            .permissions();
        parent_perms.set_mode(0o555);
        let _ = fs::set_permissions(dest_conflict.parent().unwrap(), parent_perms);
    }

    let res = deploy_skills(&[skill], &[target], &harnesses);

    // Clean up permissions so tempdir can be dropped cleanly
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut parent_perms = fs::metadata(dest_conflict.parent().unwrap())
            .unwrap()
            .permissions();
        parent_perms.set_mode(0o777);
        let _ = fs::set_permissions(dest_conflict.parent().unwrap(), parent_perms);
    }

    assert!(
        !res.errors.is_empty(),
        "Should report error on locked destination"
    );
}

#[test]
fn test_remove_skill_deletes_across_all_harness_paths() {
    let mock_target = common::create_mock_project_target();
    let proj_path = mock_target.path();

    let codex_skill = proj_path.join(".codex/skills/skill-to-delete");
    let claude_skill = proj_path.join(".claude/skills/skill-to-delete");
    fs::create_dir_all(&codex_skill).unwrap();
    fs::write(codex_skill.join("SKILL.md"), "# Delete me").unwrap();
    fs::create_dir_all(&claude_skill).unwrap();
    fs::write(claude_skill.join("SKILL.md"), "# Delete me").unwrap();

    let target = TargetConfig {
        id: "proj".to_string(),
        name: "Project".to_string(),
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

    assert!(codex_skill.exists());
    assert!(claude_skill.exists());

    remove_skill_from_target("skill-to-delete", &target, &harnesses)
        .expect("Removal should succeed");

    assert!(
        !codex_skill.exists(),
        "Codex skill folder should be removed"
    );
    assert!(
        !claude_skill.exists(),
        "Claude skill folder should be removed"
    );
}
