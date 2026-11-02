# Skill-Manager

* **Author:** Sven Kalbhenn
* **Date:** 2026-11-02

## Overview

Skill-Manager is a dark-themed desktop application built with Rust and Dioxus 0.7 for managing LLM agent skills. It discovers skills defined by a `SKILL.md` file within user-defined source folders and fans them out to target project repositories or a system-wide Global destination.

For every deployment, the application writes the complete skill folder to all configured harnesses (e.g., Codex, Claude Code, Cursor, Antigravity) by appending each harness's relative path to the target project directory (or resolving to the harness's global folder path for the Global destination). When multiple harnesses share identical relative or global paths, the destination path list is deduplicated prior to copying to prevent redundant disk operations. Overwriting existing target skills is permitted directly, as skills are authored and maintained solely in source folders, while projects are tracked in Git. Configuration is persisted in a unified, atomically written `~/.config/skill-manager/config.json` file.

## Analysis

### 1. UX & Interaction Architecture
* **File System Picker Integration**: Manually entering absolute file paths in text fields causes high error rates. Native platform folder pickers via `rfd` (Rust File Dialog) allow folder selection for source folders, target folders, and harness global paths across macOS, Linux, and Windows.
* **Three-Tab Two-Pane Structure**:
  * **Tab 1 (Source)**: Left pane displays tracked source folders (Name, Path, Exists indicator, Remove action). Right pane displays discovered skills within the selected source folder, checkboxes for multi-selection, and a "Copy Skills" trigger.
  * **Tab 2 (Targets)**: Left pane displays target folders and a pinned default "Global" target. Right pane lists installed skills across that target, rendering the names of all harnesses containing that skill in a secondary sub-label, accompanied by a "Remove Skill" action.
  * **Tab 3 (Harnesses/IDEs)**: Left pane lists defined harnesses (with a pinned "Default" harness). Right pane configures the harness relative subfolder path (e.g., `.codex/skills`) and global path (e.g., `~/.agents/skills`).
* **Direct Copy Flow**: When the user selects skills and initiates "Copy Skills", a modal allows selecting one or more targets. Confirming immediately initiates the copy to all configured harnesses. Overwriting existing skill directories in targets is performed directly without diffing or staging directories.

### 2. Runtime Execution & Storage Architecture
* **Single Configuration File (`config.json`)**: All state (`sources`, `targets`, `harnesses`) is consolidated into `~/.config/skill-manager/config.json`. This enables atomic writes (writing to a temporary file and renaming) to prevent file corruption on unexpected exits, simplifies backup, and provides unified deserialization at startup.
* **Destination Path Deduplication**: Multiple harnesses may share identical relative folder paths (e.g., both Claude Code and OpenCode pointing to `.claude/skills`) or identical global folder paths. Before copying or deleting, the engine resolves destination paths for all applicable harnesses into a deduplicated set (e.g., `HashSet<PathBuf>`). Each unique filesystem location is written or deleted exactly once.
* **Straightforward Filesystem Execution**: Agent skills typically comprise a small set of files and subdirectories. Direct synchronous filesystem calls keep the architecture lightweight without complex background task queues or thread-pool overhead.
* **Resilient Fan-Out Error Handling**: When deploying across multiple harnesses, if an error occurs on a specific harness path (e.g., permissions or path error), the application reports an error message for that harness and continues copying to all remaining harnesses.
* **Tilde (`~`) Path Resolution**: Rust's standard library does not expand `~` in `std::path::PathBuf`. Paths stored with `~` are expanded via `dirs::home_dir()` or path expansion utilities before running existence checks or I/O.
* **Directory Initialization**: The copy engine creates missing parent directory hierarchies (`create_dir_all`) automatically before copying skill trees.

### 3. Domain Model & Operational Boundaries
* **Skill Identification Invariant**: A directory is identified as a valid skill if and only if it directly contains a `SKILL.md` file. This prevents arbitrary nested directories, dotfolders, and non-skill assets from cluttering the catalog.
* **Harness Fan-Out Matrix with Deduplication**: Deploying to a project target fans out across all defined harnesses in Tab 3 with path deduplication:
  $$\text{Destinations} = \text{Unique}\left(\{ \text{Target Path} \cup \text{Harness.relative\_path} \cup \text{Skill Name} \mid \text{Harness} \in \text{Harnesses} \}\right)$$
  Deploying to the "Global" target fans out across every harness's global path with path deduplication:
  $$\text{Destinations} = \text{Unique}\left(\{ \text{Harness.global\_path} \cup \text{Skill Name} \mid \text{Harness} \in \text{Harnesses} \}\right)$$
* **Missing Global Path Handling**: If a harness lacks a configured global folder path, the application issues a warning and proceeds with deployment to all other valid global harness paths.
* **Removal Blast Radius**:
  * Tab 1 "Remove Folder": Unregisters the folder from `config.json`. Does not touch local disk files.
  * Tab 2 "Remove Skill": Permanently deletes the skill directory from every configured harness path inside that target project (or from every configured global path for the Global target), using the same deduplicated path resolution. The original source folder remains untouched.

### 4. Ecosystem Fit & Metadata Extraction
* **Frontmatter Parsing**: Parsing YAML frontmatter inside `SKILL.md` enables displaying human-readable title and description metadata alongside directory names in the UI.
* **Pre-seeded Harness Defaults**: Initializing `config.json` with standard agent harness paths (Codex, Claude Code, Cursor, Antigravity) allows the tool to be immediately functional upon initial launch without requiring manual path entry.

### 5. Implementation Layout & Modularity
* **Separation of Concerns**:
  * `models`: Structs for `AppConfig`, `Source`, `Target`, `Harness`, `Skill` with `serde` serialization.
  * `services`: Config storage (`config_store.rs`), directory scanning (`fs_scanner.rs`), and recursive file deployment/deletion with deduplication (`deployer.rs`).
  * `components`: Modular Dioxus components for each tab, modal dialogs, status badges, and folder pickers.

## Goals vs. Non-Goals

* **In-Scope Goals:**
  * Native desktop GUI in Rust using Dioxus 0.7 with dark mode styling.
  * Consolidated configuration persistence in `~/.config/skill-manager/config.json`.
  * Skill discovery constrained to immediate subfolders containing `SKILL.md`.
  * Multi-skill selection and fan-out copying to selected target projects across all defined harnesses.
  * Destination path deduplication across harnesses sharing identical relative or global paths.
  * System-wide global distribution across all defined harness global paths.
  * Automatic creation of missing parent harness directories.
  * Multi-harness status display in Tab 2, indicating all harnesses that map to the installed skill's path.
  * Synchronized multi-harness skill deletion from target projects and global locations.
  * Native folder picker dialogs (`rfd`) for directory selection.
  * Error reporting per harness during copy while completing remaining harness targets.

* **Out-of-Scope Non-Goals:**
  * SQLite database or separate multi-file config schemas.
  * Symlinking or directory junctions (incompatible with several agent harnesses).
  * In-app skill text editing or Markdown authoring.
  * In-place target updating without source overwrite ("Update Skill" button excluded; overwrites originate strictly from Tab 1).
  * Staged temporary directory rollbacks or background thread pool queues.
  * Remote Git repository cloning or package registry management (operates purely on local directories).
  * Automated conflict diffing or merging between divergent skill files.

## Alternatives Weighed

* **SQLite vs. YAML vs. Consolidated JSON**:
  * *SQLite Rejected*: Heavyweight dependencies (`rusqlite`), C bindings, schema migrations, and non-inspectable binary format provide zero benefit for small configuration lists.
  * *YAML Rejected*: Deprecated crate ecosystem in Rust (`serde_yaml` archived) and brittle parsing semantics with unquoted paths.
  * *Separate JSON Files Rejected*: Requires managing multiple file descriptors and exposes the application to partial state corruption if interrupted mid-save.
  * *Consolidated JSON Adopted*: `~/.config/skill-manager/config.json` provides simple single-file atomic persistence, straightforward backups, and direct human inspectability.
* **Unfiltered Copying vs. Pre-Copy Path Deduplication**:
  * *Rejected*: Blindly copying to every harness path sequentially causes redundant overwrites when multiple harnesses share paths (e.g., `.claude/skills`).
  * *Adopted*: Resolve and deduplicate destination paths before copying, executing the copy operation exactly once per unique destination folder.
* **Complex Staged Atomic Rollbacks vs. Direct Overwrite with Error Continuation**:
  * *Rejected*: Two-phase commit staging into temporary directories and full-target rollbacks on failure adds significant architectural complexity.
  * *Adopted*: Direct recursive copy with overwrite. Errors on individual harnesses surface a message to the user while allowing remaining harnesses to finish.
* **Asynchronous Background Task Queues vs. Direct Synchronous I/O**:
  * *Rejected*: Complex worker channels and thread pools for file copying.
  * *Adopted*: Direct synchronous file operations, keeping code minimal since skill directories contain small text files and manifests.
* **Symlinking vs. Full Directory Copying**:
  * *Rejected*: Symlinking allows instant zero-copy synchronization, but several target agent environments fail to resolve or follow symlinked skill hierarchies.
  * *Adopted*: Full directory copying with recursive file replication and clean overwrites.
* **Manual Path Text Inputs vs. Native Folder Dialogs**:
  * *Rejected*: Pure text input fields for system paths.
  * *Adopted*: Text input combined with native OS folder picker buttons (`rfd`) to prevent typo-induced path failures.

## Key Decisions & Staging for Next Steps

* **Config Storage**: Consolidated into a single `~/.config/skill-manager/config.json` with atomic write semantics.
* **Marker File**: `SKILL.md` is required for a directory to be recognized as an agent skill.
* **Path Deduplication**: Destination paths are deduplicated into a unique set before executing copy or delete operations.
* **Harness Fan-Out**: Every copy operation targets all defined harnesses. Intermediate missing directories are created automatically (`mkdir -p`).
* **Error Resilience**: If one harness path fails during copy, an error message is surfaced, and remaining harnesses proceed.
* **Deletion Semantics**: Target skill deletion wipes the skill from all configured harness paths for that target. Source folder removal in Tab 1 only modifies the config list without deleting files on disk.
* **Next Step**: Proceed to formal design document generation (`/egm-draft`).
