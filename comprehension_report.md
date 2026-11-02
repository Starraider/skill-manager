# Comprehension Test Report: Naive Reader Audit

**Target File**: `docs/designs/skill_manager.md`  
**Reviewer**: Naive Reader (`egm-comprehension`)  
**Scope**: FeatureRange (1..376) & DetailedRange (377..490)  
**Stage**: Stage 6 (Comprehension Test)

---

### What this proposes
The document proposes Skill-Manager, a native cross-platform desktop application built in Rust using Dioxus 0.7 that centralizes the discovery, deployment, and removal of LLM agent skills across multiple coding harnesses (such as Codex, Claude Code, Cursor, and Antigravity). It scans local source directories for skills identified by a `SKILL.md` marker file, parses frontmatter metadata, and enables users to fan out full directory copies simultaneously to selected target projects or a system-wide Global target. Destination paths across harnesses are deduplicated prior to copying to avoid redundant writes or lock conflicts, and parent directories are automatically created on demand. All application configurations—including sources, targets, and harness path mappings—are persistently maintained via atomic writes to a single JSON configuration file (`~/.config/skill-manager/config.json`).

### How the existing system works, per this document
Developers currently manage agent skills manually across fragmented, tool-specific relative paths in project repositories (e.g., `.codex/skills`, `.claude/skills`, `.cursor/skills`, and `.antigravity/skills`) and diverse global user paths. Keeping skills synchronized requires repetitive manual copying or custom command-line scripts, which frequently leads to outdated skills, partial copies, and a lack of visibility into which skills are installed where. When multiple agent harnesses share identical directory conventions, uncoordinated copying causes redundant overwrites and file lock conflicts, with no unified way to view installed skills or cleanly uninstall a skill across all harnesses in a single synchronized action.

---

## Findings

### 1. Missing Specification for the `scan_installed_skills` Algorithm
* **Severity**: 🟡 Major
* **Affected Scope**: Section 8.4 (Execution Flows & Subsystems) & Section 9.3 (`src/services/fs_scanner.rs`, line 411)
* **Discrepancy Description & Rationale**: Missing specification for the `scan_installed_skills` algorithm. Line 411 mentions that `fs_scanner.rs` implements `scan_installed_skills(target: &TargetConfig, harnesses: &[HarnessConfig]) -> Vec<InstalledSkill>` to discover deployed skills in project or global locations. However, Section 8.4 completely omits this subsystem from the execution flows. The document never specifies:
  1. How the scanner inspects project targets versus the Global target.
  2. What qualifies as an installed skill directory (e.g., does it require a `SKILL.md` marker file inside target harness folders, or does any child directory count?).
  3. How discoveries across disparate harness directories are reconciled and aggregated into `InstalledSkill { name, harness_names, installed_paths }`.
  Because Tab 2 (Targets & Installed Skills) and CUJ 1/3 depend on this data to display harness badges and trigger deletions, the missing algorithm creates a major implementation gap.

### 2. Missing Struct Definitions for `SkillFrontmatter` and `DeploymentResult`
* **Severity**: 🟡 Major
* **Affected Scope**: Section 8.3 (Data Models & Configuration Schema, lines 198–213) & Section 9.2–9.3 (lines 398–401, 414)
* **Discrepancy Description & Rationale**: Missing struct definitions for `SkillFrontmatter` and `DeploymentResult`. Line 400 states that `src/models/skill.rs` implements `SkillFrontmatter (parsed from SKILL.md)`, and lines 266, 303, and 414 specify that `deploy_skills` returns `DeploymentResult`. Neither struct is defined in the data models code block (lines 198–213) or in the implementation checklist. Their fields, member types, and error/status reporting structures are completely unstated.

### 3. Destination Path Canonicalization Failure on Non-Existent Directories
* **Severity**: 🟡 Major
* **Affected Scope**: Section 8.4.3 (Fan-Out Deployment & Deduplication Engine, lines 257–265) & Section 9.3 (`src/services/deployer.rs`, line 414)
* **Discrepancy Description & Rationale**: Destination path canonicalization failure on non-existent directories. Step 258 specifies: *"Map each destination path to its canonical or normalized representation. Insert into HashSet<PathBuf> to eliminate duplicates..."* prior to step 263: *"Ensure parent directory exists via std::fs::create_dir_all(dest_path.parent())"*. In standard Rust, `std::fs::canonicalize` requires paths to already exist on disk; calling it on non-existent destination paths returns `std::io::ErrorKind::NotFound`. Attempting to canonicalize target paths before calling `create_dir_all` will cause deployment to fail for any fresh project target or newly configured harness path. The document must clarify that purely lexical path normalization is used, or that parent directories must be created prior to canonicalization.

### 4. Unspecified YAML Frontmatter Parser and Missing YAML Crate in `Cargo.toml`
* **Severity**: 🟡 Major
* **Affected Scope**: Section 8.4.2 (Skill Discovery Subsystem, line 243), Section 8.5 (Alternatives Considered, line 320), and Section 9.1 (`Cargo.toml`, line 386)
* **Discrepancy Description & Rationale**: Unspecified YAML frontmatter parser and missing YAML crate in `Cargo.toml`. Lines 112, 243, and 411 state that `fs_scanner.rs` parses YAML frontmatter delimited by `---`. However, `Cargo.toml` (line 386) lists only `dioxus`, `serde`, `serde_json`, `rfd`, `dirs`, `walkdir`, `uuid`, and `thiserror`—with no YAML library included. Furthermore, Section 8.5 explicitly notes that `serde_yaml` is archived and deprecated. The document does not specify whether frontmatter is parsed using a specific alternative crate (e.g., `yaml-rust2` or `serde_yml`) or via custom manual line-splitting logic.

### 5. Ambiguous `is_pinned` Field on `HarnessConfig` vs. Unpinned `TargetConfig`
* **Severity**: 🟡 Major
* **Affected Scope**: Section 8.3 (Data Models, lines 190–196, 219–226) & Section 9.4 (Tab 2 & Tab 3 UI, lines 430–435)
* **Discrepancy Description & Rationale**: Ambiguous `is_pinned` field on `HarnessConfig` vs. unpinned `TargetConfig`. `HarnessConfig` includes `pub is_pinned: bool` (line 195), yet all pre-seeded default harnesses have `Pinned: No` (line 219), and the Tab 3 form specification (lines 432–435) does not include or explain pinning for harnesses. In contrast, the document repeatedly describes the "Global" target as "pinned" in Tab 2 (lines 226, 431), but `TargetConfig` has no `is_pinned` field (only `is_global: bool`). The design neither explains what `is_pinned` on a harness does nor clarifies whether the pinned "Global" target is protected from deletion via the "Remove Target" button.

### 6. Missing UI Triggers for Adding or Removing Harnesses in Tab 3
* **Severity**: 🟡 Major
* **Affected Scope**: Section 9.4 (`src/components/tab_harnesses.rs`, lines 432–435)
* **Discrepancy Description & Rationale**: Missing UI triggers for adding or removing harnesses in Tab 3. The specification states: *"Left pane: list of configured harnesses (Codex, Claude Code, Cursor, Antigravity, custom). Right pane: editable form fields for harness name, relative project subfolder path, and global folder path with rfd picker button."* Unlike Tab 1 ("Add Source", "Remove Source") and Tab 2 ("Add Target", "Remove Target"), Tab 3 defines no buttons or workflows for adding a new harness or removing an existing one, leaving it unexplained how "custom" harnesses are introduced into `AppConfig`.

### 7. Unstated Tilde Expansion During Destination Path Resolution
* **Severity**: 🟢 Nit
* **Affected Scope**: Section 8.4.3 (Fan-Out Deployment, lines 250–256) & Section 8.4.4 (Synchronized Removal, lines 268–275)
* **Discrepancy Description & Rationale**: Unstated tilde expansion during destination path resolution. Section 8.4.1 defines `expand_tilde` and Section 8.4.2 explicitly notes tilde expansion for source folders. However, the deployment and removal sequences do not specify expanding tildes when building destination paths from `harness.global_path` (which defaults to tilde paths like `~/.agents/skills`) or `target.path`. Passing unexpanded tilde paths to `std::fs::create_dir_all` or `std::fs::remove_dir_all` results in literal `~` directories created in the current working directory.

### 8. Undefined Application `Error` Type
* **Severity**: 🟢 Nit
* **Affected Scope**: Section 9.1 (`Cargo.toml`, line 386) & Section 9.3 (Core Engine Services, lines 408–414)
* **Discrepancy Description & Rationale**: Undefined application `Error` type. Service signatures in the implementation plan declare return types such as `Result<(), Error>` and `Result<AppConfig, Error>`, and `thiserror` is listed as a dependency in `Cargo.toml`. However, the document does not define the central error enum, its location, or its variants (e.g., I/O errors, JSON serialization errors, frontmatter parse errors).

