# Implementation Readiness Review Report: Technical Execution & Zero-Decision Audit

**Target File**: `docs/designs/skill_manager.md`  
**Reviewers**: Implementation Tech Lead (`egm-implementation-readiness`), Codebase Details Auditor (`egm-codebase-details`), Technical Copy Editor (`egm-style`)  
**Standard**: `resources/document_standards/implementation-readiness-standard.md`  
**Scope**: DetailedRange (lines 407..520)  
**Stage**: Stage 9 (Implementation Readiness Test)

---

## Reviewer Status

| Reviewer | Lens | Status |
|---|---|---|
| `egm-implementation-readiness` | Actionability, Zero-Decision Rule & Execution Instructions | Completed |
| `egm-codebase-details` | Codebase Accuracy, Dependencies, Rust/Dioxus Patterns & Test Fixtures | Completed |
| `egm-style` | Tone, Technical Precision, Redundancy & Editorial Consistency | Completed |

---

## Consolidated Findings (Blockers → Majors → Nits)

### 🔴 Blockers

#### 1. Missing Application Error Module & Concrete `AppError` Enum
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 436–445 (`src/services/config_store.rs`, `src/services/fs_scanner.rs`, `src/services/deployer.rs`)
* **Discrepancy Description**: All core service signatures declare return types with generic `Result<T, Error>` (e.g. `load_config() -> Result<AppConfig, Error>`, `scan_skills(...) -> Result<Vec<Skill>, Error>`), and `thiserror` is listed in `Cargo.toml`. However, no error module (`src/error.rs`) or concrete `AppError` enum is specified in the plan. The implementer must independently invent error variants (`Io`, `Json`, `InvalidSkillName`, `PathTraversal`, `FrontmatterError`), violating the Zero-Decision Rule.
* **Guidance**: Add `src/error.rs` to the file target checklist, define the concrete `AppError` enum using `thiserror::Error` with all specific variants, and bind service signatures to `Result<T, AppError>`.

#### 2. Missing Domain Data Models: `SkillFrontmatter` & `DeploymentResult`
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 428–430 (`src/models/skill.rs`) & Lines 442–445 (`src/services/deployer.rs`)
* **Discrepancy Description**:
  - `src/models/skill.rs` states it implements `SkillFrontmatter (parsed from SKILL.md)`, but its struct definition, field types, and optionality are omitted from the design doc.
  - `deploy_skills` returns `DeploymentResult`, which is central to reporting deployment status, counts, and error feedback across UI components, but `DeploymentResult` is nowhere defined in the models or services.
* **Guidance**: Explicitly define `SkillFrontmatter` and `DeploymentResult` in `src/models/skill.rs`:
  ```rust
  #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
  pub struct SkillFrontmatter {
      pub name: Option<String>,
      pub title: Option<String>,
      pub description: Option<String>,
  }

  #[derive(Clone, Debug, Default, PartialEq)]
  pub struct DeploymentResult {
      pub copied_paths: Vec<PathBuf>,
      pub warnings: Vec<String>,
      pub errors: Vec<(PathBuf, String)>,
  }
  ```

#### 3. Dependency Omissions and Mismatches in `Cargo.toml`
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 414–416 (`Cargo.toml`)
* **Discrepancy Description**:
  The plan lists dependencies as: `dioxus` (0.7 with desktop feature), `serde` (with derive), `serde_json`, `rfd` (0.15), `dirs` (5.0), `walkdir` (2.5), `uuid` (v4), and `thiserror`. Four required crates are missing:
  1. `tempfile`: Required for atomic persistence via `NamedTempFile` on Windows and for test fixtures (`tempfile::tempdir()`).
  2. `regex`: Required for validating `skill.name` against `^[a-zA-Z0-9_-]+$`.
  3. `tokio`: Required for `tokio::task::spawn_blocking` (with features `["rt", "rt-multi-thread"]`).
  4. Frontmatter parser crate (`serde_yml` or `yaml-rust2`) or an explicit zero-dependency line-parsing specification.
  Meanwhile, `uuid` is included despite natural path and slug keys.
* **Guidance**: Update `Cargo.toml` with concrete crate versions, feature flags, and explicit parser choices, and prune unneeded dependencies.

#### 4. Undefined Dioxus Reactive UI State, Props Contracts & Async Invocations
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 446–471 (`src/main.rs`, `src/components/*`)
* **Discrepancy Description**:
  The UI component checklist specifies visual pane layouts ("Left pane... Right pane...") but completely omits Dioxus 0.7 reactive state management:
  - No central UI state struct or navigation enum is specified (`active_tab`, `selected_source_id: Option<String>`, `selected_target_id: Option<String>`, `selected_skills: HashSet<String>`, `modal_open: bool`).
  - No pattern is documented for invoking async operations (`dioxus::spawn`) and passing reactive signals to loading indicators or error banners.
  - No component props or event callback signatures (`EventHandler<T>`) are defined between parent and child components.
* **Guidance**: Define the centralized UI state struct and navigation enum in `src/main.rs` (or `src/models/state.rs`) provided via `use_context_provider`, and specify props/event contracts for child components.

---

### 🟡 Majors

#### 5. Unpartitioned Implementation Plan & Missing Incremental Validation Checkpoints
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 407–483 ("Detailed Implementation Plan")
* **Discrepancy Description**: The Implementation Readiness Standard requires: *"The plan must partition the work into logical, incremental phases. Each phase must leave the codebase in a stable, buildable, and testable state."* The current plan groups files in a flat list by architectural layer and defers all integration tests to the end. There are no intermediate validation checkpoints (`cargo check`, `cargo test ...`) between data models, storage, services, and UI components.
* **Guidance**: Restructure into sequential, testable phases (Phase 1: Project Setup & Core Models, Phase 2: Configuration & Persistence Engine, Phase 3: Filesystem Discovery & Deployment Services, Phase 4: UI Shell & Components, Phase 5: End-to-End Integration Tests), each concluding with a concrete verification command.

#### 6. Underspecified `scan_installed_skills` Aggregation & Matching Algorithm
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 439–441 (`src/services/fs_scanner.rs`)
* **Discrepancy Description**: `scan_installed_skills(target: &TargetConfig, harnesses: &[HarnessConfig]) -> Vec<InstalledSkill>` omits the scanning and aggregation algorithm:
  - Does discovery require `SKILL.md` inside target harness subdirectories, or does any subdirectory count?
  - How are skills found across different harness folders merged into a single `InstalledSkill` instance?
  - How are missing target subdirectories handled (ignored silently vs logged)?
* **Guidance**: Specify the exact aggregation algorithm: iterate over harnesses, resolve target subpaths, verify child directories containing `SKILL.md`, group matching folder basenames into `InstalledSkill`, and populate `harness_names` and `installed_paths`.

#### 7. Contradiction on Atomic Persistence (`.tmp` rename vs `tempfile::NamedTempFile`)
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 436–438 (`src/services/config_store.rs`)
* **Discrepancy Description**: Section 8.4.1 (line 237) and Section 8.6.3 (line 403) specify atomic replacement via `tempfile::NamedTempFile` and platform-resilient overwrite to prevent Windows rename conflicts. In contrast, line 438 states: "using the atomic write pattern (write to `.tmp` file and rename)". Standard `rename` fails on Windows if the file exists.
* **Guidance**: Align line 438 to explicitly state that `tempfile::NamedTempFile` within the target configuration directory is used for atomic replacement.

#### 8. Misplaced Desktop Window Configuration (`Dioxus.toml` vs `WindowBuilder`)
* **Reviewer Lens**: `egm-codebase-details`
* **Affected Scope**: Lines 417–420 (`Dioxus.toml`) & Lines 468–471 (`src/main.rs`)
* **Discrepancy Description**: Line 419 asserts that `Dioxus.toml` "Defines application metadata, assets directory, and desktop window defaults (title: "Skill-Manager", initial size: 1100x750, min size: 900x600)." In Dioxus 0.7 Desktop, window properties cannot be configured via `Dioxus.toml`; they must be configured programmatically in `src/main.rs` via `dioxus::desktop::Config::default().with_window(WindowBuilder::new().with_title(...))`.
* **Guidance**: Relocate desktop window configuration details to `src/main.rs` and clarify that `Dioxus.toml` defines application packaging metadata.

#### 9. Abstract Verification Plan Lacking Concrete Test Functions & Mock Fixtures
* **Reviewer Lenses**: `egm-implementation-readiness`, `egm-codebase-details`
* **Affected Scope**: Lines 485–516 (`## Verification Plan`) & Lines 472–482 (`Integration Tests`)
* **Discrepancy Description**: The automated test plan specifies running `cargo test` across three test files, but provides only narrative summaries without concrete test function names, mock filesystem fixtures, or exact assertion outcomes (such as path traversal rejection, directory symlink skipping, and deduplication of overlapping paths). Additionally, tests lack a shared test helper module (e.g. `tests/common/mod.rs`) for RAII-cleaned tempdir management.
* **Guidance**: Detail specific test functions per test file with mock inputs and assertions, and specify `tests/common/mod.rs` for fixture creation.

#### 10. Borrow Checker & Thread Bounds with Asynchronous Task Dispatches
* **Reviewer Lens**: `egm-codebase-details`
* **Affected Scope**: Lines 442–445 (`src/services/deployer.rs`) & Lines 453–455 (`src/components/copy_modal.rs`)
* **Discrepancy Description**: Service signatures take borrowed slices (`deploy_skills(skills: &[Skill], targets: &[TargetConfig], harnesses: &[HarnessConfig])`), but closures dispatched to worker threads (`tokio::task::spawn_blocking` / `dioxus::spawn`) require `'static` bounds. Borrowed slices cannot cross thread boundaries without cloning into owned `Vec`s.
* **Guidance**: Clarify that UI event handlers clone relevant data into owned `Vec`s before spawning background tasks.

#### 11. Imprecise Jargon: "Synchronized Disk Deletion" vs Batch Sequential Removal
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Line 461 (`src/components/tab_targets.rs`) & Line 481 (`tests/deployer_tests.rs`)
* **Discrepancy Description**: "Synchronized" implies concurrency primitives (mutexes, locks) or filesystem flushing (`fsync`). The intended behavior is batch sequential deletion across configured harness directories.
* **Guidance**: Replace with "batch deletion across configured harness directories".

---

### 🟢 Nits

#### 12. Checklist Formatting: "Rationale" Misnomer vs "Specification"
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Lines 416, 419, 424, 427, 430, 435, 438, 441, 444, 449, 452, 455, 458, 461, 464, 467, 470, 475, 478, 481
* **Discrepancy Description**: Each checklist item uses the label `Rationale` to describe file contents and APIs rather than architectural justification ("why").
* **Guidance**: Change label from `Rationale` to `Specification` or `Details`.

#### 13. Redundant Qualification and Tautologies in Test Descriptions
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Lines 444, 478, 481, 498, 506
* **Discrepancy Description**: Phrases like "deduplicates unique", "temporary mock directory trees", and "writes to unique destinations exactly once" contain synonym redundancies.
* **Guidance**: Tighten phrasing to standard technical descriptions.

#### 14. Undefined CSS Asset File
* **Reviewer Lens**: `egm-codebase-details`
* **Affected Scope**: Line 419 & Line 467
* **Discrepancy Description**: The plan mentions "assets directory" and "dark-theme stylesheet / CSS variables", but no CSS file (e.g., `assets/style.css`) is listed in the file creation checklist.
* **Guidance**: Add `assets/style.css` to the created file checklist.

#### 15. Extraneous Document Watermark
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Line 519 (Document footer)
* **Discrepancy Description**: `*Drafted with EGM*` watermark in technical specification.
* **Guidance**: Remove footer watermark.

