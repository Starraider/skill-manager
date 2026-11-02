#![allow(dead_code)]

use std::fs;
use tempfile::TempDir;

/// Creates a temporary directory for configuration tests.
pub fn create_mock_config_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create mock config directory")
}

/// Creates a temporary directory containing sample skill directories with `SKILL.md` files.
pub fn create_mock_source_with_skills() -> TempDir {
    let dir = tempfile::tempdir().expect("Failed to create mock source directory");
    let base = dir.path();

    // Skill 1: Valid YAML frontmatter with uppercase SKILL.md
    let skill1 = base.join("git-helper");
    fs::create_dir_all(&skill1).unwrap();
    fs::write(
        skill1.join("SKILL.md"),
        "---\nname: git-helper\ntitle: Git Helper\ndescription: Streamlines git workflows\n---\n# Git Helper\nDocs here.\n",
    )
    .unwrap();
    fs::write(skill1.join("helper.sh"), "#!/bin/bash\necho helper\n").unwrap();

    // Skill 2: Lowercase skill.md
    let skill2 = base.join("code-review");
    fs::create_dir_all(&skill2).unwrap();
    fs::write(
        skill2.join("skill.md"),
        "---\nname: code-review\ntitle: Code Reviewer\ndescription: Reviews pull requests\n---\n# Code Review\n",
    )
    .unwrap();

    // Skill 3: Malformed YAML frontmatter (should fall back to folder basename)
    let skill3 = base.join("bad-yaml");
    fs::create_dir_all(&skill3).unwrap();
    fs::write(
        skill3.join("SKILL.md"),
        "---\nname: [unclosed bracket\n---\n# Bad YAML\n",
    )
    .unwrap();

    // Non-skill directory (no SKILL.md)
    let non_skill = base.join("not-a-skill");
    fs::create_dir_all(&non_skill).unwrap();
    fs::write(non_skill.join("README.md"), "# Not a skill\n").unwrap();

    dir
}

/// Creates a temporary target project directory.
pub fn create_mock_project_target() -> TempDir {
    tempfile::tempdir().expect("Failed to create mock project target directory")
}
