# Design

## Context

The repository has no CLI implementation yet. `terminal-installer-project-blueprint.md` recommends a TypeScript and Node.js CLI with terminal prompts, a typed configuration model, native filesystem operations, and a plan before writes. The behavior contract is in `specs/skill-distribution/spec.md`.

## Goals / Non-Goals

**Goals:**

- Keep AI tool definitions entirely in editable YAML after first-run seeding.
- Use one validated installation plan for global and project copies.
- Replace one skill directory safely without disturbing adjacent skills.
- Keep the terminal flow usable when a graphical folder dialog is unavailable.

**Non-Goals:**

- Managing skills inside AI tools through their APIs, marketplaces, or cloud accounts.
- Automatically updating preconfigured tool paths after the user has created a config file.
- Synchronizing installations continuously, uninstalling skills, or adding a non-interactive install mode in the first release.

## Decisions

### CLI architecture

Use TypeScript on Node.js. Commander provides the executable entry point and help, `@clack/prompts` drives the interactive flow, Zod validates the in-memory configuration, and `yaml` reads and edits the config. Use `node:fs/promises`, `node:path`, and `node:os` for filesystem and path work. Execa is needed only for platform folder-picker commands; the skill copy itself must use Node APIs. Vitest covers configuration, discovery, planning, and copy behavior with temporary directories. This follows the supplied blueprint without adding its installer-only dependencies such as Semver.

Keep modules separated by responsibility: configuration load/save and validation, source discovery, prompt orchestration, folder selection, plan construction, and plan execution. Prompts gather choices; they do not copy skill files. The planner receives validated paths and selections and emits explicit source-to-target operations before execution.

### Configuration model and defaults

Use a user-level `config.yaml`: `${XDG_CONFIG_HOME:-~/.config}/skill-manager/config.yaml` on macOS/Linux and `%APPDATA%/skill-manager/config.yaml` on Windows, with a Commander `--config` override for explicit configuration. The CLI prints the path in configuration errors so users know what to edit. Source and saved project paths are absolute after normalization; global tool paths accept absolute paths or a leading `~/`; project tool paths are relative to each saved project root. Do not expand arbitrary environment-variable expressions in YAML paths.

Example shape:

```yaml
sources: []
projects: []
tools:
  codex:
    name: Codex
    globalSkillsDir: ~/.codex/skills
    projectSkillsDir: .codex/skills
  custom-agent:
    name: Custom Agent
    globalSkillsDir: ~/custom-agent/skills
    projectSkillsDir: .custom-agent/skills
```

On the first run only, create the file with five tool entries. Current documented default pairs are:

| Tool | Global skills directory | Project skills directory | Reference |
| --- | --- | --- | --- |
| Codex | `~/.codex/skills` | `.codex/skills` | [OpenAI](https://developers.openai.com/blog/eval-skills) |
| Claude Code | `~/.claude/skills` | `.claude/skills` | [Anthropic](https://code.claude.com/docs/en/skills) |
| Antigravity | `~/.gemini/config/skills` | `.agents/skills` | [Google](https://antigravity.google/docs/skills) |
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` | [OpenCode](https://opencode.ai/docs/skills) |
| Cursor | `~/.cursor/skills` | `.cursor/skills` | [Cursor](https://prod.cursor.com/help/customization/skills) |

After creation, never merge defaults into an existing file. That makes removal of a built-in tool durable. Load every tool from the YAML map, validate both paths for each entry, and derive the menu from its valid keys. An incomplete or invalid entry produces a field-specific error before installation. Reject project paths that are absolute, empty, or escape the project root after normalization. When adding source or project paths through the CLI, update the YAML document while preserving the user's tool entries and comments, then write it atomically.

An alternative was to hard-code built-in destinations and store only overrides. That would reintroduce removed tools or require a second disabled-tools list, so the YAML map is simpler and authoritative.

### Discovery and selection

After loading or initializing the config, ask whether to add a source path as the first interactive question. Validate and save any new source before scanning. Scan configured source roots for directories containing `SKILL.md`, including nested skills, and use the containing directory name as the skill's installation name. Show each item as name plus source path. Validate `SKILL.md` presence; do not treat ordinary `.md` files as skill bundles. Keep the full bundle, including scripts and other resources, for copying.

Use Clack multi-selection for skills and AI tools. If a multi-selection contains two skills with the same directory name, explain the collision and return to skill selection until the user chooses one version. The UI displays both versions and their source paths. This avoids silently choosing a source by search order. If no skills or tools are selected, exit without a copy.

### Scope, projects, and folder picker

After tool selection, ask the global-installation question. For the project branch, ask whether to add a project, open an operating-system folder picker when one is available, and fall back to a validated path prompt on an unsupported, remote, or headless terminal. Use a small adapter for macOS, Windows, and Linux picker commands, invoked with argument arrays through Execa. Cancellation of the picker does not create a project entry. Add a chosen project to YAML before showing the project multi-select list, deduplicating normalized project paths.

An alternative was terminal-only path input. The user explicitly requested a folder dialog, so it is the fallback rather than the primary interaction.

### Copy planning and execution

Resolve all selected destinations as the cross-product of selected skills and tools, plus selected projects for project scope. Normalize target paths and deduplicate repeated source-to-target operations when tool definitions point to the same directory. Show a plan identifying new and replacement destinations before execution. Preflight that source bundles are readable, target parents can be created, and no destination is the source itself or a parent of it.

Execute replacements per target by copying the source bundle to a temporary sibling, renaming any old target to a temporary backup, and moving the staged bundle into place. Remove the backup only after success; restore it if the final move fails. This removes stale files from the old version and avoids leaving a half-copied skill. Each target is independent: a later failure does not roll back earlier successful targets, so report per-target results accurately. Do not traverse symlinks out of a skill bundle while copying.

An alternative was copying directly over an existing directory. That would leave removed source files behind and could expose an incomplete bundle if a copy fails.

## Risks / Trade-offs

- [AI tools change their discovery directories] -> Keep documented defaults in editable YAML and never reseed existing configs.
- [GUI folder pickers depend on the local desktop and installed platform commands] -> Fall back to terminal path entry and keep the picker isolated from the rest of the flow.
- [A broad source scan may find nested `SKILL.md` files users did not intend to distribute] -> Display full source paths and require explicit selection.
- [Several copies can partially succeed] -> Use per-target staging and rollback, then report each result rather than claiming run-wide atomicity.
- [Different tools can load compatible directories used by another tool] -> Default to each tool's own documented path and deduplicate only when configured paths resolve to the same directory.

## Migration Plan

There is no existing application data to migrate. On first launch, create a config with defaults; on later launches, load and validate the existing file without rewriting tool definitions. Users can roll back an installation by restoring their previous skill directories; the CLI's per-target temporary backup is for immediate failure recovery, not long-term version history.
