# Tasks

## 1. CLI foundation

- [x] 1.1 Create the TypeScript Node.js package, executable entry point, and build scripts; verify `npm run build` produces a runnable CLI.
- [x] 1.2 Add Commander, Clack, YAML, Zod, Execa, and Vitest dependencies with separate config, discovery, prompt, picker, planner, and executor modules; verify package installation and type checking succeed.

## 2. YAML configuration

- [x] 2.1 Implement platform config location and `--config`, first-run YAML creation, and the five documented tool defaults; verify a temporary first-run config has empty source/project lists and all five path pairs.
- [x] 2.2 Implement YAML loading, validation, tilde and relative-path resolution, and field-specific errors; verify tests cover malformed YAML, missing tool paths, invalid source/project paths, and project path escape attempts.
- [x] 2.3 Implement atomic persistence of newly added source and project paths while preserving tool edits and comments; verify tests show a removed default stays removed and a custom tool remains selectable after a save and restart.
- [x] 2.4 Document the config location, YAML schema, path rules, manual tool addition/removal, and `--config` in the README; verify the examples parse with the config loader.

## 3. Skill discovery and selection

- [x] 3.1 Discover directories containing `SKILL.md` under configured source roots, including nested bundles, and retain their full source paths; verify fixture tests include nested skills, supporting files, and non-skill Markdown files.
- [x] 3.2 Implement skill multi-selection with name plus source path and reject selecting two same-named skills in one run; verify prompt-level tests show both `review` entries and accept only one.
- [x] 3.3 Handle empty sources, empty discoveries, cancellation, and no skill selection without changing target directories; verify these prompt outcomes in tests.

## 4. Tool, scope, and project prompts

- [x] 4.1 Build AI tool multi-selection solely from validated YAML entries, then ask the global-or-project question; verify tests show custom entries, omit deleted defaults, and keep the requested prompt order.
- [x] 4.2 Implement platform graphical folder-picker adapters and terminal path-entry fallback; verify adapter tests cover success, cancellation, and unavailable dialog behavior without requiring a desktop in CI.
- [x] 4.3 Save newly selected project folders, show all saved projects for multi-selection, and require at least one project for project installation; verify tests cover immediate availability of a new folder and multi-project selection.
- [x] 4.4 Document the interactive flow and graphical picker fallback in the README; verify the documented flow matches an interactive smoke run with a temporary config.

## 5. Installation planning and copying

- [x] 5.1 Build a normalized source-to-target plan for global and project scopes, deduplicate shared target directories, and print new versus replacement destinations; verify planner tests cover multiple tools, multiple projects, and shared paths.
- [x] 5.2 Preflight source readability, destination safety, and target parent creation without modifying an existing skill; verify tests reject a source-to-itself copy and invalid destinations before execution.
- [x] 5.3 Copy complete bundles using a staged sibling and per-target backup, replace same-named directories, and restore the old directory on a failed final move; verify filesystem tests cover stale-file removal, unrelated-skill preservation, supporting files, and rollback.
- [x] 5.4 Report each successful or failed destination with source and target paths; verify a multi-target failure test does not report unsuccessful destinations as installed.
- [x] 5.5 Document replacement behavior and per-target failure recovery in the README; verify the documented behavior against the filesystem tests.

## 6. End-to-end verification

- [x] 6.1 Run built-CLI smoke tests with temporary source, config, and target directories for both global and multi-project installation; verify the expected skill trees and unchanged unrelated directories.
- [x] 6.2 Run the full test suite, type check, and build; verify all commands pass and the generated CLI starts with help output.
