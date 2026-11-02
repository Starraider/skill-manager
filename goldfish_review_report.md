# Goldfish Annotated Review Report

**Status:** resolved  
**Target:** `docs/designs/skill_manager.md` (DetailedRange: Lines 407–651)

## Prioritized Findings & AI Suggestions

### 1. Missing Application Error Module & Concrete `AppError` Enum [`egm-implementation-readiness`] - Severity: High
* **Affected Scope**: [Lines 436–445](docs/designs/skill_manager.md#L436-L445) (`src/services/config_store.rs`, `src/services/fs_scanner.rs`, `src/services/deployer.rs`)
* **Discrepancy Description**: All service method signatures specify generic `Result<T, Error>`, and `thiserror` is listed in `Cargo.toml`. However, no error module (`src/error.rs`) or concrete `AppError` enum is declared in the checklist, forcing the implementer to independently invent error variants.
* **Rationale**: The Zero-Decision Rule requires all error types and variants to be explicitly specified in the plan.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Essential for clean error propagation across storage, scanning, and deployment.
* ***Proposed Edit:***
  * Location: Implementation Checklist Phase 1 (Data Models)
  * Change: Add `src/error.rs` defining `pub enum AppError` with variants: `Io(#[from] std::io::Error)`, `Json(#[from] serde_json::Error)`, `InvalidPath(PathBuf)`, `TraversalAttempt(String)`, `Frontmatter(String)`, and bind service signatures to `Result<T, AppError>`.

---

### 2. Missing Domain Data Models: `SkillFrontmatter` & `DeploymentResult` [`egm-implementation-readiness`, `egm-codebase-details`] - Severity: High
* **Affected Scope**: [Lines 428–430](docs/designs/skill_manager.md#L428-L430) (`src/models/skill.rs`) & [Lines 442–445](docs/designs/skill_manager.md#L442-L445) (`src/services/deployer.rs`)
* **Discrepancy Description**: `SkillFrontmatter` is referenced without a field definition or serde attributes, and `DeploymentResult` is returned by `deploy_skills` but is completely undefined in the models or checklist.
* **Rationale**: Complete data structures are prerequisites for implementing scanning and deployment services.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Eliminates implementer ambiguity regarding status reporting and frontmatter deserialization.
* ***Proposed Edit:***
  * Location: `src/models/skill.rs`
  * Change: Explicitly define `SkillFrontmatter` and `DeploymentResult`:
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

---

### 3. Dependency Omissions and Mismatches in `Cargo.toml` [`egm-codebase-details`] - Severity: High
* **Affected Scope**: [Lines 414–416](docs/designs/skill_manager.md#L414-L416) (`Cargo.toml`)
* **Discrepancy Description**: `Cargo.toml` is missing four required crates: `tempfile` (for atomic persistence and test fixtures), `regex` (for skill name sanitization), `tokio` (for `tokio::task::spawn_blocking`), and a YAML parser (`yaml-rust2` or `serde_yml`). Meanwhile, `uuid` is included despite natural path and slug keys.
* **Rationale**: Dependencies must be exact to guarantee clean compilation without developer guesswork.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Aligns the package manifest with the architectural requirements.
* ***Proposed Edit:***
  * Location: `Cargo.toml` dependency specification
  * Change: Specify dependencies: `dioxus = { version = "0.7", features = ["desktop"] }`, `serde = { version = "1.0", features = ["derive"] }`, `serde_json = "1.0"`, `yaml-rust2 = "0.8"`, `rfd = "0.15"`, `dirs = "5.0"`, `walkdir = "2.5"`, `regex = "1.10"`, `tempfile = "3.10"`, `thiserror = "1.0"`, and `tokio = { version = "1.38", features = ["rt", "rt-multi-thread"] }`. Remove `uuid`.

---

### 4. Undefined Dioxus Reactive UI State, Props Contracts & Async Invocations [`egm-implementation-readiness`] - Severity: High
* **Affected Scope**: [Lines 446–471](docs/designs/skill_manager.md#L446-L471) (`src/main.rs`, `src/components/*`)
* **Discrepancy Description**: The UI component checklist specifies visual pane layouts but completely omits Dioxus 0.7 reactive state management, signal types, props contracts, and async task execution patterns.
* **Rationale**: Implementers must know the reactive architecture and component interfaces before writing UI code.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Centralized UI state prevents signal scattering and defines clear props interfaces.
* ***Proposed Edit:***
  * Location: `src/models/state.rs` (or `src/main.rs`) and `src/components/*`
  * Change: Define `AppState` holding `Signal<AppConfig>`, `Signal<ActiveTab>`, `Signal<Option<String>>` for selections, `Signal<HashSet<String>>` for skill checkboxes, and `Signal<bool>` for modal visibility. Specify props and `EventHandler` callbacks for child components.

---

### 5. Unpartitioned Implementation Plan & Missing Incremental Validation Checkpoints [`egm-implementation-readiness`, `egm-codebase-details`] - Severity: Medium
* **Affected Scope**: [Lines 407–483](docs/designs/skill_manager.md#L407-L483) ("Detailed Implementation Plan")
* **Discrepancy Description**: The plan lists files in a flat architectural layout and defers all integration tests to the end without incremental compilation or testing checkpoints between phases.
* **Rationale**: Phased execution ensures each phase leaves the codebase in a stable, buildable, and testable state.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Phased partitioning with checkpoints (`cargo check`, `cargo test ...`) enables disciplined, verified execution.
* ***Proposed Edit:***
  * Location: Section `## Detailed Implementation Plan`
  * Change: Restructure into 5 sequential phases:
    1. Phase 1: Package Manifests & Domain Models (Verification: `cargo check`).
    2. Phase 2: Configuration & Persistence Engine (Verification: `cargo test --test config_store_tests`).
    3. Phase 3: Filesystem Discovery & Deployment Services (Verification: `cargo test --test fs_scanner_tests --test deployer_tests`).
    4. Phase 4: UI Shell, Layout & Feature Tabs (Verification: `cargo check`).
    5. Phase 5: Integration Verification & Manual Matrix (Verification: `cargo test`).

---

### 6. Underspecified `scan_installed_skills` Aggregation & Matching Algorithm [`egm-codebase-details`] - Severity: Medium
* **Affected Scope**: [Lines 439–441](docs/designs/skill_manager.md#L439-L441) (`src/services/fs_scanner.rs`)
* **Discrepancy Description**: `scan_installed_skills(target: &TargetConfig, harnesses: &[HarnessConfig]) -> Vec<InstalledSkill>` omits the discovery and aggregation logic.
* **Rationale**: The algorithm must be documented to ensure consistent discovery of installed skills across multiple harness directories.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Unambiguous scanning logic guarantees correct badge display in Tab 2.
* ***Proposed Edit:***
  * Location: `src/services/fs_scanner.rs`
  * Change: Specify the algorithm: iterate over configured harnesses, resolve project or global subpaths, verify immediate child directories containing `SKILL.md` (case-insensitively), group matching directory basenames into `InstalledSkill`, and populate `harness_names` and `installed_paths`.

---

### 7. Contradiction on Atomic Persistence (`.tmp` rename vs `tempfile::NamedTempFile`) [`egm-codebase-details`] - Severity: Medium
* **Affected Scope**: [Lines 436–438](docs/designs/skill_manager.md#L436-L438) (`src/services/config_store.rs`)
* **Discrepancy Description**: Line 438 specifies manual write to `.tmp` and rename, contradicting Section 8.4.1 which mandates `tempfile::NamedTempFile` and platform-resilient replacement for Windows safety.
* **Rationale**: Prevent runtime crashes on Windows when overwriting existing configuration files.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Harmonizes implementation details with the architecture specification.
* ***Proposed Edit:***
  * Location: `src/services/config_store.rs`
  * Change: Update line 438 to specify using `tempfile::NamedTempFile` within the target configuration directory followed by `persist`.

---

### 8. Misplaced Desktop Window Configuration (`Dioxus.toml` vs `WindowBuilder`) [`egm-codebase-details`] - Severity: Medium
* **Affected Scope**: [Lines 417–420](docs/designs/skill_manager.md#L417-L420) (`Dioxus.toml`) & [Lines 468–471](docs/designs/skill_manager.md#L468-L471) (`src/main.rs`)
* **Discrepancy Description**: Line 419 claims window properties are set in `Dioxus.toml`, whereas Dioxus 0.7 Desktop configures window title and dimensions programmatically in `src/main.rs`.
* **Rationale**: Prevent developer confusion from non-functional configuration keys.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Direct programmatic window configuration matches Dioxus 0.7 Desktop standards.
* ***Proposed Edit:***
  * Location: `Dioxus.toml` and `src/main.rs`
  * Change: Specify that `Dioxus.toml` configures application packaging metadata, and document `dioxus::desktop::Config::default().with_window(...)` in `src/main.rs`.

---

### 9. Abstract Verification Plan Lacking Concrete Test Functions & Mock Fixtures [`egm-implementation-readiness`] - Severity: Medium
* **Affected Scope**: [Lines 485–516](docs/designs/skill_manager.md#L485-L516) (`## Verification Plan`) & [Lines 472–482](docs/designs/skill_manager.md#L472-L482)
* **Discrepancy Description**: The test plan provides only narrative descriptions without concrete test function names, mock filesystem fixtures, or exact assertions.
* **Rationale**: The Zero-Decision Rule requires actionable, executable test specifications.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Specific test names and assertions provide clear implementation targets.
* ***Proposed Edit:***
  * Location: `## Verification Plan` and test checklist
  * Change: Add `tests/common/mod.rs` for mock fixtures, and detail concrete test functions for `config_store_tests.rs`, `fs_scanner_tests.rs`, and `deployer_tests.rs`.

---

### 10. Borrow Checker & Thread Bounds with Asynchronous Task Dispatches [`egm-codebase-details`] - Severity: Medium
* **Affected Scope**: [Lines 442–445](docs/designs/skill_manager.md#L442-L445) (`src/services/deployer.rs`) & [Lines 453–455](docs/designs/skill_manager.md#L453-L455) (`src/components/copy_modal.rs`)
* **Discrepancy Description**: Service signatures take borrowed slices, but closures passed to `tokio::task::spawn_blocking` or `dioxus::spawn` require `'static` bounds.
* **Rationale**: Borrowed slices cannot cross thread boundaries without cloning into owned `Vec`s.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Clarifies ownership transfer for background tasks.
* ***Proposed Edit:***
  * Location: `src/components/copy_modal.rs` and `src/components/tab_targets.rs`
  * Change: Specify that UI event handlers clone data into owned `Vec<Skill>`, `Vec<TargetConfig>`, and `Vec<HarnessConfig>` before dispatching background tasks.

---

### 11. Imprecise Jargon: "Synchronized Disk Deletion" vs Batch Sequential Removal [`egm-style`] - Severity: Medium
* **Affected Scope**: Line 461 (`src/components/tab_targets.rs`) & Line 481 (`tests/deployer_tests.rs`)
* **Discrepancy Description**: "Synchronized" implies concurrency primitives (mutexes, locks) or filesystem flushing (`fsync`).
* **Rationale**: Precise technical phrasing avoids misleading implementers into adding unneeded locks.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Neutral technical terminology.
* ***Proposed Edit:***
  * Location: Lines 461 and 481
  * Change: Replace "synchronized disk deletion" with "batch deletion across configured harness directories".

---

### 12. Checklist Formatting: "Rationale" Misnomer vs "Specification" [`egm-style`] - Severity: Low
* **Affected Scope**: Lines 416, 419, 424, 427, 430, 435, 438, 441, 444, 449, 452, 455, 458, 461, 464, 467, 470, 475, 478, 481
* **Discrepancy Description**: Uses `Rationale` to describe file contents and APIs rather than architectural justification.
* **Rationale**: Accurate technical labeling.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Editorial consistency.
* ***Proposed Edit:***
  * Location: Implementation Checklist items
  * Change: Replace `Rationale` label with `Specification`.

---

### 13. Redundant Qualification and Tautologies in Test Descriptions [`egm-style`] - Severity: Low
* **Affected Scope**: Lines 444, 478, 481, 498, 506
* **Discrepancy Description**: Redundancies such as "deduplicates unique", "temporary mock directory trees", and "writes to unique destinations exactly once".
* **Rationale**: Technical conciseness.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Editorial tightening.
* ***Proposed Edit:***
  * Location: Lines 444, 478, 481, 498, 506
  * Change: Simplify phrasing to standard technical descriptions.

---

### 14. Undefined CSS Asset File [`egm-codebase-details`] - Severity: Low
* **Affected Scope**: Line 419 & Line 467
* **Discrepancy Description**: References stylesheet and assets directory, but `assets/style.css` is omitted from created files checklist.
* **Rationale**: Explicit file target coverage.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Completes the file manifest.
* ***Proposed Edit:***
  * Location: Implementation Checklist Phase 4
  * Change: Add `assets/style.css` to created files list.

---

### 15. Extraneous Document Watermark [`egm-style`] - Severity: Low
* **Affected Scope**: Line 519 (Document footer)
* **Discrepancy Description**: `*Drafted with EGM*` watermark.
* **Rationale**: Professional document standards.
* ***AI Suggestion:*** PROPOSE FIX
* ***Justification:*** Clean document formatting.
* ***Proposed Edit:***
  * Location: Line 519
  * Change: Delete footer watermark.
