# Skill-Manager Design Document

*   **Author:** Sven Kalbhenn
*   **Status:** Draft
*   **Date:** 2026-11-02

--------------------------------------------------------------------------------

## Overview

Skill-Manager is a desktop application built in Rust using Dioxus 0.7 for managing and distributing LLM agent skills. Development workflows frequently rely on multiple AI coding assistants—such as Codex, Claude Code, Cursor, and Antigravity—each maintaining distinct configuration and skill directory conventions. Skill-Manager centralizes skill discovery from local source repositories and automates distribution across all target projects and global agent environments.

The application discovers skills strictly identified by a `SKILL.md` marker file in immediate source subdirectories. When a user deploys skills to a project target or the system-wide Global target, Skill-Manager fans out full directory copies to all configured harnesses. Destination paths are deduplicated prior to execution so shared folders receive exactly one write. Application configuration is maintained in a single, atomically written configuration file (`~/.config/skill-manager/config.json`).

--------------------------------------------------------------------------------

## The Problem

Developers using agentic AI tools face high friction and fragmentation:

1. **Fragmented Directory Conventions**: Each AI harness uses different relative directories in project repositories (e.g., `.codex/skills`, `.claude/skills`, `.cursor/skills`, `.antigravity/skills`) and different global user paths (e.g., `~/.agents/skills`, `~/.claude/skills`).
2. **Manual and Error-Prone Replication**: Keeping custom skills updated across multiple projects and harnesses requires repetitive manual file copying or command-line scripting, resulting in outdated or partially copied skills.
3. **Overlapping Path Redundancy**: Multiple harnesses sometimes share identical folder paths (such as multiple tools standardizing on `.claude/skills` or shared global directories). Naive copy scripts perform redundant overwrites or encounter file lock conflicts.
4. **Lack of Centralized Visibility**: There is no straightforward way to view which skills are installed in a given project, which harnesses are currently configured with those skills, or to cleanly remove a skill across all harnesses in a single action.

Skill-Manager solves this by providing a unified, dark-themed desktop interface to scan source directories, view discovered skills, fan out copies to project or global targets with automatic path deduplication, and inspect or delete installed skills consistently across all harnesses.

--------------------------------------------------------------------------------

## Goals

*   **Native Cross-Platform Desktop UI**: Deliver a responsive desktop interface using Rust and Dioxus 0.7 with dark mode styling and native OS directory pickers via `rfd`.
*   **Strict Skill Discovery**: Automatically detect valid agent skills within configured source folders by requiring an immediate child directory containing a `SKILL.md` marker file.
*   **Multi-Harness Fan-Out**: Support selecting multiple skills and copying them simultaneously to selected target projects across all defined harnesses.
*   **Destination Path Deduplication**: Resolve and deduplicate destination paths into unique canonical locations before executing copy or deletion operations to avoid redundant I/O.
*   **System-Wide Global Distribution**: Provide a built-in "Global" target that deploys skills across every harness's configured global folder path.
*   **Missing Directory Initialization**: Automatically create required parent directory structures (`create_dir_all`) for each target harness path prior to file copying.
*   **Synchronized Removal**: Enable removing an installed skill from a target project or global location, deleting it from all associated harness paths in a single action while leaving source directories untouched.
*   **Resilient Error Handling**: Report errors encountered on specific harness paths without halting deployment to remaining valid harness targets.
*   **Atomic Configuration Storage**: Maintain all sources, targets, and harness definitions in `~/.config/skill-manager/config.json` using atomic write operations (temp file write followed by atomic rename).

--------------------------------------------------------------------------------

## Non-Goals

*   **Database Systems**: No inclusion of SQLite, embedded key-value engines, or multi-file configuration stores; configuration remains strictly within a single JSON file.
*   **Symlinks and Directory Junctions**: Symlinking is excluded due to compatibility failures across various agent CLI harnesses; all deployments are direct full directory copies.
*   **In-App Skill Authoring**: No built-in Markdown editing, syntax linting, or skill authoring tools; authoring remains in user text editors and IDEs.
*   **Remote Repositories & Registries**: No Git cloning, remote downloading, or public registry package management; operations are strictly local filesystem operations.
*   **In-Place Target Updates Without Source**: No mechanism to edit or update installed skills directly inside target harness directories; source directories remain the single source of truth.
*   **Two-Phase Staged Rollback**: No staging directories or complex transactional rollback mechanisms; copy operations overwrite target folders directly.
*   **Diffing and Conflict Merging**: No visual diffing or interactive merge resolution when overwriting existing target skill folders.

--------------------------------------------------------------------------------

## Open Questions

*   **Question 1 (I/O Concurrency vs UI Thread)**: Should recursive file copying and directory scanning execute on a background worker thread (`tokio::task::spawn_blocking` or `std::thread`) to ensure the Dioxus UI loop never stutters during operations on large trees or slow drives?
*   **Question 2 (Discovery Depth)**: Does strict immediate-subfolder discovery (`source_folder/<skill>/SKILL.md`) suffice for all user workflows, or should a bounded search depth (e.g., depth 2) be supported for nested skill repositories?

--------------------------------------------------------------------------------

## Critical User Journeys (CUJs)

### CUJ 1: Discover and Deploy Skills to a Target Project

*   **Given**: The user has added a source directory containing `git-helper` (with `git-helper/SKILL.md`) and a target project directory `~/projects/my-app`. Two harnesses are active: Codex (`.codex/skills`) and Claude Code (`.claude/skills`).
*   **When**: The user opens Tab 1, selects `git-helper`, clicks "Copy Skills", checks `~/projects/my-app`, and confirms the dialog.
*   **Then**: Skill-Manager resolves the two harness destination paths, creates parent directories if missing, recursively copies the skill directory into `~/projects/my-app/.codex/skills/git-helper` and `~/projects/my-app/.claude/skills/git-helper`, and displays `git-helper` under `my-app` in Tab 2 with both harness labels.

### CUJ 2: Deploy Skills Globally Across All Harnesses

*   **Given**: Harnesses configured with global paths (e.g., Codex at `~/.agents/skills`, Claude Code at `~/.claude/skills`).
*   **When**: The user selects `git-helper` in Tab 1, clicks "Copy Skills", and selects the pinned "Global" target.
*   **Then**: The application deduplicates the destination paths, copies the full skill directory to each unique global directory, and reports a warning for any harness missing a configured global path while successfully completing the others.

### CUJ 3: Remove Installed Skill from Target Project

*   **Given**: Target project `~/projects/my-app` contains `git-helper` deployed across multiple harness folders.
*   **When**: The user navigates to Tab 2, locates `git-helper` under `my-app`, and clicks "Remove Skill".
*   **Then**: The application resolves all harness subpaths for `git-helper` inside `my-app`, deletes each directory from disk, and removes the skill entry from the target's installed list in the UI without modifying the source folder.
