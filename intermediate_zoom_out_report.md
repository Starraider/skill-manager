# Intermediate Zoom Out Review Report: Post-Mitigation Architecture Audit

**Target File**: `docs/designs/skill_manager.md`  
**Reviewer**: Skeptical Architect (`egm-zoom-out`)  
**Standard**: `resources/document_standards/zoom-out-standard.md`  
**Prior Report**: `zoom_out_report.md`  
**Stage**: Stage 8 (Intermediate Zoom Out Review)

---

## Executive Summary

The additions applied after the Comprehension and Critic review stages introduce several commendable, minimalist risk mitigations—specifically regarding symlink safety (`walkdir.follow_links(false)`), file lock handling (per-file error capture without rollback complexity), and case-insensitive discovery. Offloading I/O to asynchronous tasks via `dioxus::spawn` correctly prevents UI thread freezing without over-engineering a custom background task queue.

However, the revised design introduces a significant architectural anti-pattern and security contradiction around **path traversal sanitization**: by attempting to use user-authored `SKILL.md` frontmatter `name` as the destination directory name instead of the source directory's basename, the design creates a self-inflicted path traversal risk and patches it with regex validation and fallback mechanics. In addition, the **sentinel value anti-pattern** (`TargetConfig.is_global = true` with empty `PathBuf`) was not resolved and directly compromises the newly added deletion/copy boundary containment checks. Finally, several newly specified mitigations rely on external crates (`regex`, `tempfile`, `tokio`, `serde_yaml`) that are missing from `Cargo.toml` while speculative dependencies (`uuid`) remain.

---

## Findings

### 1. Sentinel Value Anti-Pattern in `TargetConfig` Compromises Boundary Containment Checks
* **Severity**: 🔴 Blocker
* **Affected Scope**: Lines 183–189, Line 228, Lines 254–261, and Lines 268, 278–279
* **Simplicity Rationale**:
  The revised design retains `TargetConfig { path: PathBuf::new(), is_global: true }`. This sentinel value directly undermines the newly added deletion and copy boundary checks (`dest_path.starts_with(&base_dir)`):
  - For standard project targets, `base_dir` is `target.path`.
  - For the "Global" target, `target.path` is empty (`""`). In Rust path arithmetic, `path.starts_with("")` is an invalid or vacuous check, forcing `deployer` and `fs_scanner` to implement dual-branch boundary verification logic.
  - Illegal states remain representable: any function that accepts a `TargetConfig` and operates on `target.path` without an explicit `if target.is_global` guard operates on an invalid empty path.
  *Simpler Alternative*: Model deployment destinations idiomatic to Rust using an algebraic type (e.g. `enum TargetKind { Project(PathBuf), Global }`) or separate Global distribution from project targets in the domain model.

---

### 2. Destination Renaming via Frontmatter Incurs Path Traversal Vulnerability & Over-Engineered Sanitization
* **Severity**: 🟡 Major
* **Affected Scope**: Lines 245–247, Lines 257–261, Lines 277–281, and Line 393
* **Simplicity Rationale**:
  The design attempts to parse `skill.name` from `SKILL.md` YAML frontmatter and use that string as the destination folder name on disk (`target.path.join(harness.relative_path).join(&skill.name)`).
  This introduces a self-inflicted path traversal vulnerability, which the design then patches with complex regex validation (`^[a-zA-Z0-9_-]+$`), fallback logic to directory basenames, and secondary containment checks (`dest.starts_with(&base_dir)`).
  This also causes operational desynchronization:
  - If a source folder `git-helper` defines `name: custom-helper` in frontmatter, it deploys as `custom-helper`. The source folder name and deployed folder name no longer match.
  - If the author later changes `name` in `SKILL.md`, subsequent deployments create orphaned folders in target projects.
  - In agent conventions (Codex, Claude Code, Cursor, Antigravity), skill folders are standard directory trees identified directly by their directory name.
  *Simpler Root-Cause Solution*: Use the source directory's basename (`folder_path.file_name()`) as the immutable destination folder name. Operating systems already guarantee that directory basenames cannot contain `/`, `\`, or traversal tokens (`..`). This eliminates the path traversal risk at the root, removes the need for regex validation or fallbacks, guarantees 1:1 folder name parity between source and target, and keeps frontmatter parsing purely display-oriented.

---

### 3. Async I/O Layer Boundary Ambiguity & Missing `tokio` Dependency
* **Severity**: 🟡 Major
* **Affected Scope**: Lines 43, 250, 360–366, 414–416, and 444
* **Simplicity Rationale**:
  Offloading I/O to background tasks via `dioxus::spawn` is the right architectural choice to keep the UI responsive without building a custom task queue. However, the revised specification introduces layer confusion and dependency discrepancies:
  - The text prescribes `tokio::task::spawn_blocking` (lines 43, 250, 361), but `tokio` is omitted from `Cargo.toml` (line 416).
  - Line 115 labels the engine subsystem as `deployer.rs (async background)`, while line 444 specifies synchronous signatures `deploy_skills(...) -> DeploymentResult`.
  *Simpler Architecture*: Keep the filesystem service modules (`fs_scanner.rs`, `deployer.rs`) as pure, synchronous, standard-library Rust functions with zero async or runtime dependencies. Asynchronous execution should be handled exclusively at the UI component layer (`components/`) by wrapping calls to these synchronous services in `dioxus::spawn` (or background threads). This maintains a clean architectural boundary and avoids tangling the core engine with Tokio or Dioxus runtimes.

---

### 4. Dependency Discrepancies and Omissions for New Mitigations
* **Severity**: 🟡 Major
* **Affected Scope**: Lines 414–420 vs. Lines 237, 245–246, 257
* **Simplicity Rationale**:
  Several newly specified mitigations rely on external crates that are missing from `Cargo.toml`:
  - Regex validation (`regex`) for skill name traversal sanitization (lines 246, 257).
  - Atomic persistence via `tempfile::NamedTempFile` (lines 237, 403).
  - Asynchronous thread offloading via `tokio::task::spawn_blocking` (line 250).
  - YAML frontmatter parsing (`serde_yaml` or equivalent) (line 245).
  Simultaneously, `uuid (v4)` remains in `Cargo.toml` (line 416) despite sources and project targets having natural unique paths and harnesses having natural unique slug keys.
  The manifest should be pruned of unneeded dependencies (`uuid`) and either explicitly include what is truly needed or replace them with standard library primitives (e.g., standard directory basename usage instead of regex; canonical `PathBuf` keys instead of UUIDs; standard atomic rename instead of `tempfile`).

---

### 5. Commendation: Pragmatic & Minimalist Risk Mitigations
* **Severity**: 🟢 Nit (Positive Assessment)
* **Affected Scope**: Lines 369–387 (Section "Risks & Mitigations")
* **Simplicity Rationale**:
  The new "Risks & Mitigations" section successfully adheres to the Zoom Out Standard by avoiding over-engineering:
  - *Active File Locks*: Simple per-file error capture with user notifications rather than complex locking daemons or rollback transactions.
  - *Mid-Operation Partial Writes*: Leverages copy idempotency and pre-flight checks rather than heavy two-phase staging across filesystem mounts.
  - *Symlink Traversal*: Minimalist mitigation using `walkdir::WalkDir::new(...).follow_links(false)` rather than bespoke cycle detection or graph traversal logic.
  - *Case Sensitivity*: Simple case-insensitive matching for `SKILL.md` without complex filesystem normalization engines.

---

### 6. Persistent Altitude Drops in Detailed Implementation Plan
* **Severity**: 🟢 Nit
* **Affected Scope**: Lines 411–471 (Section "Detailed Implementation Plan")
* **Simplicity Rationale**:
  The implementation plan still specifies low-level code mechanics (e.g., exact helper signatures like `expand_tilde(path: &Path) -> PathBuf`, UI pane column divisions, and method return types). These micro-details belong in task-level implementation rather than an architectural design document.

