# Critic Review Report: Multi-Agent Antagonistic Audit

**Target File**: `docs/designs/skill_manager.md`  
**Reviewers**: Design Critic (`egm-design`), Structural Layout Validator (`egm-structural`), Technical Copy Editor (`egm-style`)  
**Standard**: `resources/document_standards/feature-spec-standard.md`  
**Scope**: FeatureRange (lines 1..376)  
**Stage**: Stage 7 (Critic Review)

---

## Reviewer Status

| Reviewer | Lens | Status |
|---|---|---|
| `egm-design` | Architecture, Security, Scalability & Failure Modes | Completed |
| `egm-structural` | Heading Hierarchy, Blueprint Compliance & Markdown Syntax | Completed |
| `egm-style` | Tone, Conciseness, Objectivity & Editorial Precision | Completed |

---

## Consolidated Findings (Blockers → Majors → Nits)

### 🔴 Blockers

#### 1. Arbitrary Directory Traversal, Overwrite, and Deletion
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Section "Execution Flows & Subsystems" (lines 243–275), Section "Security & Privacy" (lines 362–366)
* **Discrepancy Description**: `fs_scanner.rs` extracts `skill.name` directly from YAML frontmatter in `SKILL.md` without validation. Neither `fs_scanner.rs` nor `deployer.rs` sanitizes `skill.name` against path traversal (`../`, `..\\`) or absolute paths.
  - In deployment, paths are formed via `target.path.join(&harness.relative_path).join(&skill.name)`. In Rust, `Path::join` replaces the base if an argument is absolute; if `skill.name` traverses upward, files can be written anywhere.
  - In removal, `remove_skill_from_target` calls `std::fs::remove_dir_all(path)`. If `skill.name` is empty, `.`, or traverses upward, it will recursively delete the entire project root (`target.path`) or global harness root (`~/.agents/skills`).
* **Recommendation**:
  - Enforce strict validation on `skill.name` (e.g. regex `^[a-zA-Z0-9_-]+$`), rejecting separators or traversal tokens.
  - Enforce that `HarnessConfig.relative_path` is strictly relative (`is_relative()`) with no parent traversal components.
  - Assert that all resolved destination paths reside strictly within the target root boundary (`dest.starts_with(&base_dir)`) before copying or deleting.

#### 2. Main UI Thread Freeze & Crash Risk from Synchronous I/O
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Section "Open Questions" (lines 57–58), Section "Alternatives Considered" (lines 349–357), Section "Performance & Latency" (lines 367–371)
* **Discrepancy Description**: The spec rejects background execution by claiming skill payloads are "negligible (<500 KB)" and operations finish in "<10 ms." Real-world agent skills frequently include virtual environments (`.venv`), `node_modules`, or binary dependencies with thousands of files. Additionally, target paths may be on network shares (SMB/NFS), cloud-synced folders (iCloud/OneDrive/Dropbox), or WSL mounts. Synchronously running recursive filesystem traversal and fan-out copying on the main Dioxus/Tao window event loop freezes the UI, triggering OS beachballs/hangs and preventing cancellation.
* **Recommendation**: Offload recursive scanning, copying, and deletion to background worker tasks (`dioxus::spawn` / `tokio::task::spawn_blocking`). Provide non-blocking status updates and progress states.

#### 3. Missing Mandatory Section: "Risks & Limits" / "Risks & Mitigations"
* **Reviewer Lens**: `egm-structural`
* **Affected Scope**: Document Scope (expected lines 360–376)
* **Discrepancy Description**: The Feature Specification Standard mandates: *"The specification must explicitly call out its own constraints. It must identify potential failure modes, security implications, scalability limits, and how they will be mitigated."* Lines 1..376 contain no section titled `Risks & Limits` or `Risks & Mitigations`. Non-Functional Requirements only list aspirational goals, omitting operational failure modes and systematic mitigations.
* **Recommendation**: Add a dedicated `## Risks & Mitigations` section explicitly enumerating failure modes (e.g., partial mid-copy write failures, active file locks from running IDEs/agents, cyclic symlinks, case-sensitivity differences) and their mitigations.

---

### 🟡 Majors

#### 4. Windows Atomic Save Failure in `std::fs::rename`
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Section "1. Configuration & Tilde Resolution" (lines 230–237), Section "Rollout & Rollback" (lines 372–374)
* **Discrepancy Description**: Atomic saving writes to `config.json.tmp` and renames to `config.json` via `std::fs::rename`. On Windows, standard `std::fs::rename` maps to `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING`, returning `io::ErrorKind::AlreadyExists` when `config.json` already exists. After initial setup, all subsequent config saves will fail on Windows.
* **Recommendation**: Use a platform-resilient atomic write pattern or crate (such as `tempfile`'s `NamedTempFile::persist`) that handles Windows replacement semantics cleanly.

#### 5. Incompatible `canonicalize` Call on Non-Existent Destination Paths
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Section "3. Fan-Out Deployment & Deduplication Engine" (lines 257–260), Section "Alternatives Considered" (lines 331–339)
* **Discrepancy Description**: The spec states paths are deduplicated by mapping to a "canonical or normalized representation" before insertion into `HashSet<PathBuf>`. `std::fs::canonicalize` requires paths to already exist on disk, failing with `NotFound` if any segment does not exist. Because missing target directories are created *after* deduplication (line 263), canonicalization will fail on all fresh projects or missing harness folders.
* **Recommendation**: Use purely lexical path normalization (normalizing `.` and `..` without disk queries) or canonicalize only the existing base target directory and append normalized relative paths.

#### 6. Host Data Leak and Infinite Recursion from Source Symlinks
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Section "3. Fan-Out Deployment & Deduplication Engine" (lines 261–266), Section "Non-Goals" (lines 47–48)
* **Discrepancy Description**: Non-Goal 2 excludes creating symlinks, but does not address *existing* symlinks inside source skills. If a source skill contains symlinks pointing to parent directories or sensitive host paths (`~/.ssh`, `/etc`), a recursive copy following symlinks can leak host data into project repos or cause infinite recursion on circular links.
* **Recommendation**: Explicitly specify that directory traversal disables following symlinks (`follow_links(false)`) and skips or safely copies symlink entries without traversing.

#### 7. Standard Section Nomenclature & Hierarchy Deviations
* **Reviewer Lens**: `egm-structural`
* **Affected Scope**: Lines 17, 30, 44, 84
* **Discrepancy Description**: Three core sections deviate from the required template layout:
  - Line 17 uses `## The Problem` instead of required `## Problem Statement`.
  - Lines 30 & 44 split `Goals & Non-Goals` into separate H2 headers rather than a unified `## Goals & Non-Goals` header with `### Goals` and `### Non-Goals` subsections.
  - Line 84 uses `## Technical Plan (Architecture)` instead of the required `## Proposed Architecture`.
* **Recommendation**: Align headings to the standard template (`## Problem Statement`, `## Goals & Non-Goals`, and `## Proposed Architecture`).

#### 8. Pejorative and Loaded Language in Problem and Alternatives Framing
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Section "The Problem" (line 23), Section "Alternatives Considered" (lines 334, 338, 340, 347, 356), Section "Non-Goals" (line 51)
* **Discrepancy Description**: Phrasing such as "Naive copy scripts", "Blindly copying", "Sequential Blind Copying", and qualifying rejected designs as "Complex Staged Rollback" or "complex background queues" uses subjective, strawman framing.
* **Recommendation**: Replace loaded terms with objective descriptors (e.g., "Uncoordinated copy scripts", "Sequential un-deduplicated copying", "Two-Phase Staged Rollback", "Background Task Queue") and evaluate concrete operational trade-offs neutrally.

#### 9. Cosmetic Marketing Copy in Core Engineering Goals
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Section "The Problem" (line 26), Section "Goals" (line 32)
* **Discrepancy Description**: Calling out "dark-themed desktop interface" in the problem solution summary and "dark mode styling" in system-level engineering goals mixes cosmetic UI theming into primary architectural requirements.
* **Recommendation**: Move theming details to UI presentation specifications and focus system goals on functional capabilities.

---

### 🟢 Nits

#### 10. Representing Global Target with Empty Path (`path: ""`) and Missing `is_pinned`
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Lines 182–187, 226
* **Discrepancy Description**: Setting `TargetConfig.path` to `""` causes accidental joins to resolve against current working directory. Additionally, `TargetConfig` lacks `is_pinned` to protect the Global target from deletion.
* **Recommendation**: Add `is_pinned: bool` to `TargetConfig` and model the path safely using an enum or `Option<PathBuf>`.

#### 11. Marker File Case-Sensitivity and YAML Error Fallback
* **Reviewer Lens**: `egm-design`
* **Affected Scope**: Lines 241–246
* **Discrepancy Description**: Exact match on `SKILL.md` behaves differently on case-sensitive Linux vs case-insensitive macOS/Windows. Behavior on invalid YAML frontmatter is unspecified.
* **Recommendation**: Specify case matching rules and ensure YAML parse errors log a warning and fallback to folder basename without aborting.

#### 12. Missing Code Block Identifiers and Formatting Inconsistencies
* **Reviewer Lens**: `egm-structural`
* **Affected Scope**: Lines 90–130, 136–156, list indentation, and horizontal rules
* **Discrepancy Description**: ASCII diagrams lack `text` tags on code fences; list indentation alternates between 3-space and 1-space; 80-character hyphens are used instead of standard `---`.
* **Recommendation**: Tag code blocks with `text`, standardize on single-space bullet markers (`* `), and use `---` for rules.

#### 13. Vague Buzzwords and Synonym Redundancies
* **Reviewer Lens**: `egm-style`
* **Affected Scope**: Lines 13, 19, 32, 35, 36, 39, 245, 274, 318, 320, 328, 345, 363, 373
* **Discrepancy Description**: Terms like "high friction", "system-wide Global", "unique canonical", "Fallback name defaults to", "cleanly", "completely", "safely", and "No telemetries" introduce buzzwords, padding, and redundancies.
* **Recommendation**: Tighten prose to plain, direct technical statements as detailed in the copy audit.

