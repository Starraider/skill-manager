# Agent Skills Filesystem Storage & Discovery Specification

**Document Version:** 1.0.0  
**Date:** 2026-11-03  
**Status:** Grounded Engineering Specification & Research Report  
**Target Application:** Skill-Manager (Cross-Platform Agent Skill Synchronizer & Deployment Harness)  
**Author:** Principal Developer Tooling Architect & Systems Researcher  

---

## 1. Executive Summary Table

The table below outlines the verified filesystem discovery paths for Agent Skills across major AI coding assistants, harnesses, and IDEs.

> **Skill-Manager Precedence Rule:** In accordance with the *Standard Path Preference Principle*, whenever an AI harness natively supports the vendor-neutral `.agents/skills` directory, that standard path is designated as the primary default in Skill-Manager. This maximizes cross-tool interoperability and prevents redundant disk writes.

| Tool / IDE | Tool-Specific Project Path | Tool-Specific Global Path | Supports `<project>/.agents/skills`? | Supports `~/.agents/skills`? | Preferred Skill-Manager Project Path | Preferred Skill-Manager Global Path | Verification Status |
| :--- | :--- | :--- | :---: | :---: | :--- | :--- | :--- |
| **Cursor** | `.cursor/skills/` | `~/.cursor/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Claude Code** | `.claude/skills/` | `~/.claude/skills/` | No | No | `.claude/skills` | `~/.claude/skills` | [Verified Official] |
| **Google Antigravity** | `.agents/skills/` *(native)* | `~/.gemini/config/skills/` | **Yes** *(Primary)* | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **OpenAI Codex** | `.agents/skills/` *(native)* | `~/.agents/skills/` *(native)* | **Yes** *(Primary)* | **Yes** *(Primary)* | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **OpenCode** | `.opencode/skills/` | `~/.config/opencode/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Source-Code Confirmed] |
| **GitHub Copilot** | `.github/skills/` | `~/.copilot/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Windsurf** | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Pi (`pi-coding-agent`)** | `.pi/skills/` | `~/.pi/agent/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Cline** | `.cline/skills/` | `~/.cline/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Qoder** | `.qoder/skills/` | `~/.qoder/skills/` | Incomplete / Fallback | No | `.qoder/skills` | `~/.qoder/skills` | [Verified Official] |
| **Roo Code** | `.roo/skills/` | `~/.roo/skills/` | **Yes** | **Yes** | `.agents/skills` | `~/.agents/skills` | [Verified Official] |
| **Continue.dev** | `.continue/rules/` *(flat)* | `~/.continue/` | No | No | `.continue/skills` *(custom)* | `~/.continue/skills` | [Not Natively Supported] |
| **Aider** | `.aider/` *(flat)* | None | No | No | `.aider/skills` *(wrapper)* | `None` | [Not Natively Supported] |

---

## 2. Deep-Dive Tool Profiles

---

### 2.1 Cursor (Cursor IDE)

#### Overview & Skill Architecture

Cursor (developed by Anysphere) supports modular **Agent Skills** adhering to the open `SKILL.md` format. In Cursor's Agent chat mode, skills leverage progressive disclosure: only the metadata (`name` and `description` from the YAML frontmatter) is ingested into the system prompt index at initialization (~100 tokens per skill). When user intent matches the trigger description, or when the user explicitly invokes the skill via `/skill-name`, the full `SKILL.md` body and any on-demand support references are mounted dynamically into the conversation context.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.cursor/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
3. **Cross-Compatibility Fallbacks:** `<project-root>/.claude/skills/<skill-name>/SKILL.md`, `<project-root>/.codex/skills/<skill-name>/SKILL.md`

- **Precedence Order:** When identical skill names exist across multiple project folders, `.cursor/skills` takes precedence, followed by `.agents/skills`, then legacy `.claude/skills` / `.codex/skills`. Nested skill directories in repository subtrees are automatically scoped to that specific subtree.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.cursor/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.cursor/skills/<skill-name>/`, `~/.agents/skills/<skill-name>/`, or `$XDG_CONFIG_HOME/cursor/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.cursor\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** `CURSOR_CONFIG_DIR` (overrides the default `~/.cursor` base path).

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively. Scanned automatically on workspace load.
- **Global Standard:** Supported natively. Skills placed in `~/.agents/skills/` are surfaced across all Cursor workspaces.
- **Collision / Precedence:** If both `.cursor/skills/my-skill` and `.agents/skills/my-skill` exist, Cursor prioritizes `.cursor/skills/my-skill`. Distinct skills from both folders are merged seamlessly into the active skill registry.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Because Cursor fully supports both project and global `.agents/skills` paths, deploying to `.agents/skills` allows skills to be shared immediately with OpenAI Codex, Antigravity, OpenCode, Windsurf, Copilot, Cline, Pi, and Roo Code without duplicate writes. `.cursor/skills` remains documented as the proprietary fallback.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name` (required, lowercase ASCII, hyphen-separated), `description` (required, actionable trigger text).
- **Cursor Extensions:** `paths` (array of globs scoping skill activation to specific file paths), `disable-model-invocation: true` (restricts invocation exclusively to explicit `/name` slash commands).
- **Subdirectories:** Supports `scripts/`, `references/`, `templates/`, and `assets/`.

#### Sources & Evidence Citations

- [Cursor Documentation - Agent Skills](https://docs.cursor.com/context/skills)
- [Cursor 2.4 Changelog & Release Notes](https://cursor.com/changelog/2-4)

---

### 2.2 Claude Code (Anthropic CLI)

#### Overview & Skill Architecture

Claude Code is Anthropic's official terminal-based agentic CLI. Anthropic pioneered the `SKILL.md` specification to solve context-window saturation. Claude Code operates via strict progressive disclosure: skill metadata (~100 tokens) is loaded into the prompt header, while full markdown instructions, dynamic shell substitutions, and hooks execute on demand when triggered by semantic matching or explicit `/name` commands.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.claude/skills/<skill-name>/SKILL.md`
2. **Plugin Distribution:** `<project-root>/.claude/plugins/<plugin-name>/skills/<skill-name>/SKILL.md`

- **Precedence Order:** Project-level skills in `.claude/skills/` override plugin-provided skills. Personal/global skills in `~/.claude/skills/` override project skills of the same name.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.claude/skills/<skill-name>/`
- **Linux:** `~/.claude/skills/<skill-name>/` (or `$XDG_CONFIG_HOME/claude/skills/<skill-name>/`)
- **Windows:** `%USERPROFILE%\.claude\skills\<skill-name>\`
- **Environment Overrides:** `CLAUDE_CONFIG_DIR` (redirects the configuration directory away from `~/.claude`).

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** **Not Natively Supported.** Claude Code hardcodes discovery to the `.claude/` namespace. It will not read `<project-root>/.agents/skills` unless explicitly symlinked or packaged as an installed plugin.
- **Global Standard:** **Not Natively Supported.** Claude Code will not read `~/.agents/skills/`.
- **Collision / Precedence:** N/A (Ignored by the runtime).

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.claude/skills`
- **Global Path:** `Some("~/.claude/skills")`
- **Justification:** Because Claude Code strictly restricts its discovery scan to `.claude/skills`, Skill-Manager **must** use the harness-specific path to ensure skills are recognized without requiring user-side filesystem symlinking or custom plugin wrappers.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description`.
- **Claude Extensions:** `when_to_use`, `argument-hint`, `arguments`, `disable-model-invocation`, `allowed-tools`, `disallowed-tools`, `model`, `effort`, `hooks`, dynamic bash injection via `$ARGUMENTS` and Claude syntax.
- **Packaging:** Flat directory or bundled within a Claude Plugin containing a `plugin.json` manifest.

#### Sources & Evidence Citations

- [Claude Code Documentation - Extend Claude with Skills](https://code.claude.com/docs/en/skills)
- [Anthropic Agent Skills Specification](https://agentskills.io/specification)

---

### 2.3 Google Antigravity (Antigravity IDE & CLI Ecosystem)

#### Overview & Skill Architecture

Google Antigravity is Google's next-generation agentic pair-programming ecosystem, comprising the Antigravity IDE and the `agy` CLI. Antigravity was built from the ground up around the open `.agents` standard. It discovers instructions, rules, and skills hierarchically by walking up from the current working directory to the Git repository root. It employs progressive disclosure: YAML frontmatter is indexed, and Markdown instructions are mounted on demand.

#### Project-Level Paths & Precedence

1. **Primary Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
2. **Alternative Recognized Roots:** `<project-root>/.agent/skills/`, `<project-root>/_agents/skills/`, `<project-root>/_agent/skills/`
3. **CLI-Specific Root:** Antigravity CLI (`agy`) accepts `<project-root>/.agent/skills/` (singular) alongside `.agents/skills/`.

- **Precedence Order:** Workspace project hierarchy (CWD up to `.git` root) > Declared configurations in `skills.json` / `plugins.json` > Global user configuration > Built-in application customizations.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.gemini/config/skills/<skill-name>/` (and open standard `~/.agents/skills/<skill-name>/`)
- **Linux:** `~/.gemini/config/skills/<skill-name>/` (or `$XDG_CONFIG_HOME/gemini/config/skills/`)
- **Windows:** `%USERPROFILE%\.gemini\config\skills\<skill-name>\` (or `%USERPROFILE%\.agents\skills\`)
- **Antigravity CLI Global:** `~/.gemini/antigravity-cli/skills/`
- **Environment Overrides:** `ANTIGRAVITY_CONFIG_DIR`, `GEMINI_CLI_HOME`.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** **Native Primary Path.** Antigravity natively designates `.agents/skills` as its primary project customization directory.
- **Global Standard:** Supported natively by the Antigravity IDE runtime alongside `~/.gemini/config/skills/`.
- **Collision / Precedence:** Project skills in `.agents/skills` override global skills. When multiple matching skill files are found, deduplication is enforced by resolved canonical filesystem paths.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** `.agents/skills` is the canonical native path for Antigravity projects. Using `~/.agents/skills` globally ensures unified cross-tool sharing while being natively recognized by the IDE. `~/.gemini/config/skills` is documented as an alternative.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name` (lowercase-hyphenated string), `description` (mandatory, actionable trigger text).
- **Subdirectories:** Strongly recommends `references/` for progressive documentation, `scripts/` for executable helpers, `examples/`, and `resources/`.

#### Sources & Evidence Citations

- [Antigravity Skills Documentation](https://antigravity.google/docs/skills)
- [Antigravity Customizations System Guide (`agy-customizations`)](file:///Users/svenkalbhenn/.gemini/antigravity/builtin/skills/agy-customizations/SKILL.md)
- [Google Developer Codelab - Getting Started with Antigravity Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)

---

### 2.4 OpenAI Codex (Codex CLI / Agent Harnesses)

#### Overview & Skill Architecture

OpenAI Codex (terminal agent and IDE extensions) co-designed and standardizes on the open Agent Skills architecture. Codex traverses directory hierarchies from the launch working directory upward to the repository root. Codex evaluates the `SKILL.md` frontmatter for automatic matching or explicit `$skill-name` invocation, listing available skills via the `/skills` CLI interactive command.

#### Project-Level Paths & Precedence

1. **Primary Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
2. **Legacy / Alternative Path:** `<project-root>/.codex/skills/<skill-name>/SKILL.md`

- **Precedence Order:** Local directories closer to the current working directory take precedence over parent directories up to the repository root. `.agents/skills` is the official modern standard; `.codex/skills` is treated as a legacy fallback.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.agents/skills/<skill-name>/` (System admin: `/etc/codex/skills/`)
- **Linux:** `~/.agents/skills/<skill-name>/` (System admin: `/etc/codex/skills/`)
- **Windows:** `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** `CODEX_HOME` (defaults to `~/.codex` or `~/.agents`).

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** **Native Primary Standard.** Official documentation and CLI source specify `.agents/skills` as the project-level root.
- **Global Standard:** **Native Primary Standard.** User-level skills reside in `~/.agents/skills/`.
- **Collision / Precedence:** Skills closer to CWD override parent/global skills. Modern `.agents/skills` overrides legacy `.codex/skills`.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** `.agents/skills` is the definitive native path for Codex. Setting this avoids legacy `.codex/skills` fragmentation.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description`.
- **Codex Extensions:** Optional `agents/openai.yaml` inside the skill directory defining presentation metadata (`display_name`, `short_description`, `default_prompt`) and execution policies (`allow_implicit_invocation: true|false`).

#### Sources & Evidence Citations

- [OpenAI Developers - Agent Skills in Codex](https://developers.openai.com/codex/skills)
- [OpenAI Curated Skills Catalog](https://github.com/openai/skills)
- [OpenAI Codex CLI Repository](https://github.com/openai/codex)

---

### 2.5 OpenCode (Anomaly / OpenCode AI Terminal Assistant)

#### Overview & Skill Architecture

OpenCode is an open-source autonomous terminal-based AI coding assistant created by Anomaly. OpenCode executes skills through a native `skill` tool call. It reads `SKILL.md` frontmatter at startup and dynamically mounts the markdown body into conversation context. Permission policies (e.g., auto-executing scripts vs. requiring confirmation) are managed externally in `opencode.json`.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.opencode/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
3. **Compatibility Path:** `<project-root>/.claude/skills/<skill-name>/SKILL.md`

- **Precedence Order:** OpenCode scans up to the Git worktree root: `.opencode/skills/` > `.agents/skills/` > `.claude/skills/`. Custom paths can also be added via `opencode.json` (`skills: ["./custom-skills"]`).

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.config/opencode/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.config/opencode/skills/<skill-name>/` (respects `$XDG_CONFIG_HOME/opencode/skills/`) or `~/.agents/skills/<skill-name>/`
- **Windows:** `%APPDATA%\opencode\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** `OPENCODE_CONFIG_DIR`, `XDG_CONFIG_HOME`.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively out-of-the-box.
- **Global Standard:** Supported natively out-of-the-box (`~/.agents/skills/`).
- **Collision / Precedence:** Discovered locations are merged into a unified registry. If two skills share an identical `name`, `.opencode/skills` overrides `.agents/skills`.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** OpenCode actively parses `.agents/skills`. Utilizing the standard path enables seamless sharing across other installed assistants. `.opencode/skills` is retained in documentation as a proprietary option.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** Strictly supports the open standard: `name`, `description`, `license`, `compatibility`, `metadata`. Unknown vendor fields (e.g. Claude hooks or Cursor globs) are safely ignored.
- **Permissions:** Configured via `opencode.json` under `"permission": { "skill": { ... } }`.

#### Sources & Evidence Citations

- [OpenCode Documentation - Agent Skills](https://opencode.ai/docs/skills)
- [OpenCode Repository & Configuration Specification](https://github.com/opencode-ai/opencode)

---

### 2.6 GitHub Copilot (VS Code Extension & Agent Mode)

#### Overview & Skill Architecture

GitHub Copilot (in VS Code Agent Mode, Copilot CLI, and GitHub Copilot extensions) supports modular Agent Skills (`SKILL.md`) alongside prompt templates (`.prompt.md`), custom agents (`.agent.md`), and repository instructions (`.github/copilot-instructions.md`). Copilot uses progressive disclosure to inject skill instructions only when conversational context matches the skill's description, or via `/skill-name` commands.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.github/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
3. **Compatibility Path:** `<project-root>/.claude/skills/<skill-name>/SKILL.md`

- **Precedence Order:** `.github/skills/` takes precedence over `.agents/skills/`, which in turn takes precedence over `.claude/skills/`.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.copilot/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.copilot/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.copilot\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** VS Code User Data / Profile extension storage directory.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively in VS Code Copilot agent mode.
- **Global Standard:** Supported natively via `~/.agents/skills/`.
- **Collision / Precedence:** Project `.github/skills` overrides `.agents/skills` on naming collisions; non-colliding skills are merged into the command palette.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Conforms to the Standard Path Preference Rule. Copilot discovers `.agents/skills` natively, avoiding duplicate installations in `.github/skills`.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name` (maps to `/command`), `description` (drives model selection).
- **Subdirectories:** Supports relative links to `scripts/` and `references/`.

#### Sources & Evidence Citations

- [GitHub Copilot Documentation](https://docs.github.com/copilot)
- [VS Code Copilot Agent Skills Specification](https://code.visualstudio.com/docs/copilot/agent-skills)

---

### 2.7 Windsurf (Codeium Cascade)

#### Overview & Skill Architecture

Windsurf is Codeium's AI-native IDE powered by the Cascade agent. Cascade combines tool-calling flows with modular Agent Skills. Windsurf loads skills progressively: indexing YAML frontmatter at startup and dynamically loading instructions, project patterns, and reference documents when triggered by task context or explicit user request.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.windsurf/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`
3. **Manifest Configuration:** `<project-root>/.windsurf/skills.json`

- **Precedence Order:** `.windsurf/skills/` > `.agents/skills/`. Explicit declarations in `.windsurf/skills.json` take priority.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.codeium/windsurf/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.codeium/windsurf/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.codeium\windsurf\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** `WINDSURF_HOME`, `CODEIUM_HOME`.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively. Cascade scans `.agents/skills` as an open-standard project root.
- **Global Standard:** Supported natively via `~/.agents/skills/`.
- **Collision / Precedence:** Proprietary `.windsurf/skills/` overrides `.agents/skills/` if skills share the same name; otherwise merged.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Standard path rule applies. Storing skills in `.agents/skills` allows Windsurf Cascade to use them simultaneously with other assistants without duplicate disk usage.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description`.
- **Subdirectories:** Supports `references/` and `scripts/`.

#### Sources & Evidence Citations

- [Codeium Windsurf Official Documentation](https://codeium.com/windsurf)
- [Windsurf Agent Skills Guidelines](https://docs.codeium.com/windsurf/skills)

---

### 2.8 Pi (`pi-coding-agent`)

#### Overview & Skill Architecture

Pi (`@earendil-works/pi-coding-agent`) is a lightweight, extensible terminal coding assistant and agent harness implementing the open Agent Skills standard. Pi separates heavyweight TypeScript engine extensions from lightweight, portable `SKILL.md` packages. It discovers skills across global and project paths, exposing them via progressive disclosure and slash command invocation.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.pi/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`

- **Precedence Order:** Project skills require workspace trust before activation. Once trusted, `.pi/skills/` takes precedence over `.agents/skills/`.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.pi/agent/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.pi/agent/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.pi\agent\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** `PI_CODING_AGENT_DIR` (overrides `~/.pi/agent`, placing skills in `$PI_CODING_AGENT_DIR/skills/`).

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively out-of-the-box.
- **Global Standard:** Supported natively out-of-the-box (`~/.agents/skills/`).
- **Collision / Precedence:** Skills are merged; project `.pi/skills` overrides identical names in `.agents/skills`.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Pi explicitly supports `.agents/skills`. Standard path preference rule applies.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name` (required), `description` (required).
- **Packaging:** Can be distributed via npm/git packages (`pi install npm:@scope/pkg`) or dropped directly into skill folders.

#### Sources & Evidence Citations

- [Pi Coding Agent Documentation](https://pi.dev/docs/skills)
- [Pi Agent GitHub Repository](https://github.com/earendil-works/pi-coding-agent)

---

### 2.9 Cline (VS Code Extension)

#### Overview & Skill Architecture

Cline (formerly Claude Dev) is an autonomous coding agent for VS Code. In addition to `.clinerules` (which provide standing project-wide instructions), modern versions of Cline support modular Agent Skills (`SKILL.md`). Cline indexes skill frontmatter (~100 tokens) at startup and uses a dedicated internal skill-loading tool to pull full instruction bodies into context upon semantic trigger or `/name` execution.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.cline/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`

- **Precedence Order:** Workspace `.cline/skills/` > `.agents/skills/`. Both take precedence over user-global skills.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.cline/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.cline/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.cline\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** VS Code Global Storage path for extension state.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively.
- **Global Standard:** Supported natively (`~/.agents/skills/`).
- **Collision / Precedence:** Merged into available skills index. In case of name collision, `.cline/skills` takes priority.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Standard path rule applies. Using `.agents/skills` allows Cline to share installed skills with Cursor, Codex, Copilot, and Windsurf.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description`, optional `allowed-tools`.
- **Subdirectories:** Supports `scripts/`, `references/`, `assets/`.

#### Sources & Evidence Citations

- [Cline Official Documentation](https://cline.bot/docs/skills)
- [Cline Open Source Repository](https://github.com/cline/cline)

---

### 2.10 Qoder (IDE & CLI)

#### Overview & Skill Architecture

Qoder (encompassing Qoder IDE and Qoder CLI) is an agentic coding environment featuring specialized subagents (Explore, Plan, General-Purpose) and reusable skills. Skills teach the agent multi-step procedures (e.g. database migrations, pull request reviews) and can be triggered automatically by semantic relevance or explicitly via `/name` commands.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.qoder/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** Recognized only in selected variants/versions; not consistently discovered across all platforms.

- **Precedence Order:** Project-level `.qoder/skills` overrides user-level `~/.qoder/skills`. In CLI sessions, skills can be hot-reloaded via `/skills reload`.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.qoder/skills/<skill-name>/` *(Note: QoderWork uses `~/.qoderwork/skills/`)*
- **Linux:** `~/.qoder/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.qoder\skills\<skill-name>\`
- **Environment Overrides:** `QODER_HOME`.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** **Incomplete / Fallback Only.** Reliable discovery requires `.qoder/skills/`.
- **Global Standard:** **Not Natively Supported.** Qoder scans `~/.qoder/skills/` exclusively for user-level skills.
- **Collision / Precedence:** `.qoder/skills` takes absolute precedence.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.qoder/skills`
- **Global Path:** `Some("~/.qoder/skills")`
- **Justification:** Because Qoder does not consistently support `.agents/skills` across its IDE, CLI, and cloud products, assigning the tool-specific path `.qoder/skills` is required to ensure dependable discovery.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description` (adhering to open standard length and character constraints).
- **Subdirectories:** Supports `references/`, `scripts/`, and `templates/`.

#### Sources & Evidence Citations

- [Qoder CLI Skills Documentation](https://docs.qoder.com/en/cli/Skills)
- [Qoder IDE Extensions & Skills](https://docs.qoder.com/ja/extensions/skills)

---

### 2.11 Roo Code (VS Code Extension)

#### Overview & Skill Architecture

Roo Code (an evolution/fork of Cline) provides an autonomous agent with configurable personas/modes (`.roomodes`), standing instructions (`.roorules`), and modular Agent Skills (`SKILL.md`). Skills are loaded dynamically via progressive disclosure, minimizing baseline prompt overhead while supplying specialized domain procedures on demand.

#### Project-Level Paths & Precedence

1. **Tool-Specific Primary:** `<project-root>/.roo/skills/<skill-name>/SKILL.md`
2. **Open Standard Path:** `<project-root>/.agents/skills/<skill-name>/SKILL.md`

- **Precedence Order:** `.roo/skills/` takes precedence over `.agents/skills/`. Project skills override user-global skills.

#### Global / Shared Paths & Environment Overrides

- **macOS:** `~/.roo/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Linux:** `~/.roo/skills/<skill-name>/` or `~/.agents/skills/<skill-name>/`
- **Windows:** `%USERPROFILE%\.roo\skills\<skill-name>\` or `%USERPROFILE%\.agents\skills\<skill-name>\`
- **Environment Overrides:** VS Code storage conventions.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** Supported natively out-of-the-box.
- **Global Standard:** Supported natively out-of-the-box (`~/.agents/skills/`).
- **Collision / Precedence:** Merged; `.roo/skills` overrides `.agents/skills` on naming conflicts.

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.agents/skills`
- **Global Path:** `Some("~/.agents/skills")`
- **Justification:** Standard path rule applies.

#### Format Requirements & Packaging Conventions

- **Entry File:** `SKILL.md` (mandatory).
- **Frontmatter Fields:** `name`, `description`.
- **Subdirectories:** Supports `scripts/`, `references/`, `assets/`.

#### Sources & Evidence Citations

- [Roo Code Official Documentation](https://docs.roocode.com)
- [Roo Code GitHub Repository](https://github.com/RooVetGit/Roo-Code)

---

### 2.12 Continue.dev

#### Overview & Skill Architecture

Continue.dev is a widely-used open-source coding extension for VS Code and JetBrains. **Architectural Limitation:** Continue does **not** natively implement the autonomous progressive-disclosure Agent Skills directory paradigm (`SKILL.md` discovery). Instead, Continue uses:

- Custom slash commands configured in `config.yaml` / `config.json`.
- Prompt templates stored in `.continue/prompts/*.prompt`.
- Standing markdown rules stored in `.continue/rules/*.md`.

#### Project-Level Paths & Precedence

- `.continue/rules/` (hierarchical markdown rules injected into context).
- `.continue/prompts/` (slash command templates).
- `<project-root>/.agents/skills`: **Not Natively Supported.** Community setups load files from skill directories via HTTP context providers or custom slash commands.

#### Global / Shared Paths & Environment Overrides

- **macOS/Linux:** `~/.continue/`
- **Windows:** `%USERPROFILE%\.continue\`

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** [Not Natively Supported]
- **Global Standard:** [Not Natively Supported]

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.continue/skills` *(custom / bridge destination)*
- **Global Path:** `Some("~/.continue/skills")`
- **Justification:** Retained for users employing custom Continue context providers or bridging scripts, but flagged as lacking native autonomous progressive disclosure.

#### Sources & Evidence Citations

- [Continue.dev Custom Commands & Rules Documentation](https://docs.continue.dev/customization/prompts)

---

### 2.13 Aider (CLI Assistant)

#### Overview & Skill Architecture

Aider is a pure terminal pair-programming assistant. **Architectural Limitation:** Aider does **not** support dynamic directory-based Agent Skills or progressive disclosure. Aider is configured via:

- `.aider.conf.yml` (YAML configuration).
- `CONVENTIONS.md` (project conventions file).
- The `--read` command-line argument to load external files into chat context.
Community orchestrators wrap Aider by dynamically reading files from a skills directory into `--read`.

#### Standard `.agents/skills` Support (Project & Global)

- **Local Project Standard:** [Not Natively Supported]
- **Global Standard:** [Not Natively Supported]

#### Preferred Path Recommendation for Skill-Manager

- **Project Path:** `.aider/skills` *(wrapper / staging)*
- **Global Path:** `None`
- **Justification:** Flagged as not supporting native Agent Skills.

#### Sources & Evidence Citations

- [Aider Documentation - Configuration and Conventions](https://aider.chat/docs/config.html)

---

## 3. Edge Cases, Collisions & Differences

### 3.1 Shared Directories & Multi-Tool Deduplication

A key benefit of the vendor-agnostic `.agents/skills` directory is multi-tool interoperability:

- If a developer uses **Cursor**, **Codex**, **OpenCode**, **GitHub Copilot**, **Windsurf**, and **Cline** within the same repository, placing a skill in `.agents/skills/<name>/` makes it instantly accessible to all six tools.
- Skill-Manager's deployment engine uses hash-based and path-based deduplication (`dest_map` in `deployer.rs`). When multiple harnesses map to `.agents/skills`, Skill-Manager performs a single write operation rather than duplicating identical directories into `.cursor/skills`, `.windsurf/skills`, `.cline/skills`, etc.

### 3.2 Precedence and Collision Resolution

When a skill exists in both a tool-specific folder (e.g., `.cursor/skills/deploy-db`) and the standard folder (`.agents/skills/deploy-db`):

- All dual-path tools (Cursor, OpenCode, Copilot, Windsurf, Cline, Roo Code) resolve the collision by giving precedence to their **tool-specific proprietary path**, masking the version in `.agents/skills`.
- If the skills have different names, the runtime merges both sets into a unified skill index.
- *Recommendation for Skill-Manager:* When a user opts to standardize on `.agents/skills`, existing proprietary folders for that skill should be archived or removed to prevent stale instruction masking.

### 3.3 Windows-Specific Path & Filesystem Considerations

- **Path Separators:** While Windows APIs accept forward slashes `/`, CLI harnesses and certain VS Code extensions on Windows fail to resolve relative symlinks constructed with forward slashes. Skill-Manager must utilize `PathBuf` component joining to guarantee platform-native separators (`\`).
- **Path Length Restrictions (`MAX_PATH`):** Deeply nested skill structures (`.agents/skills/<name>/references/...`) can exceed the 260-character `MAX_PATH` on Windows unless LongPaths are enabled in the Windows Registry (`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled = 1`).
- **Symlink Privileges on Windows:** In contrast to macOS and Linux, creating directory symlinks on Windows requires administrative privileges or Windows Developer Mode enabled. Skill-Manager's current copy-based deployment model (skipping directory symlinks in `walkdir`) prevents security traversal issues and avoids Windows privilege failures.

### 3.4 Subtree Scoping (Cursor & OpenCode)

- In large monorepos, Cursor and OpenCode support subtree-scoped skills: placing `.cursor/skills/` or `.agents/skills/` inside a subfolder (e.g., `packages/backend/.agents/skills/`) scopes the contained skills strictly to interactions touching that subfolder.
- Global skills (`~/.agents/skills/`) remain active across all workspaces on the machine.

---

## 4. Ready-to-Use Skill-Manager Rust Configuration

The following Rust implementation updates `AppConfig::default_seeded()` in [`src/models/config.rs`](file:///Users/svenkalbhenn/Documents/Projekte/skill-manager/src/models/config.rs).

It strictly adheres to the **Standard Path Preference Rule**:

- Every tool supporting `<project-root>/.agents/skills` has `relative_path` assigned to `PathBuf::from(".agents/skills")`.
- Every tool supporting `~/.agents/skills` has `global_path` assigned to `Some(PathBuf::from("~/.agents/skills"))`.
- Tool-specific proprietary paths are documented inline as alternatives.
- Tools lacking `.agents/skills` support (Claude Code, Qoder) default to their verified proprietary paths.

```rust
use crate::services::i18n::Language;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub left_pane_width: Option<f64>,
    pub sources: Vec<SourceConfig>,
    pub targets: Vec<TargetConfig>,
    pub harnesses: Vec<HarnessConfig>,
}

impl AppConfig {
    pub fn default_seeded() -> Self {
        Self {
            language: Language::English,
            left_pane_width: None,
            sources: Vec::new(),
            targets: vec![TargetConfig {
                id: "global".to_string(),
                name: "Global".to_string(),
                path: PathBuf::new(),
                is_global: true,
                is_pinned: true,
            }],
            harnesses: vec![
                // 1. OpenAI Codex
                // Native Standard: Uses .agents/skills for project and ~/.agents/skills globally.
                // Alternative proprietary/legacy: .codex/skills
                HarnessConfig {
                    id: "codex".to_string(),
                    name: "OpenAI Codex".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 2. Claude Code (Anthropic CLI)
                // Proprietary: Claude Code strictly scans .claude/ and ~/.claude/.
                // Does NOT natively discover .agents/skills.
                HarnessConfig {
                    id: "claude-code".to_string(),
                    name: "Claude Code".to_string(),
                    relative_path: PathBuf::from(".claude/skills"),
                    global_path: Some(PathBuf::from("~/.claude/skills")),
                    is_pinned: false,
                },

                // 3. Cursor (Cursor IDE)
                // Dual Support: Natively supports .agents/skills as well as .cursor/skills.
                // Preferred standard path selected to enable multi-agent sharing.
                // Alternative proprietary: .cursor/skills, ~/.cursor/skills
                HarnessConfig {
                    id: "cursor".to_string(),
                    name: "Cursor".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 4. Google Antigravity (Antigravity IDE & CLI)
                // Native Standard: Antigravity workspace customizations default to .agents/skills.
                // Alternative proprietary: .agent/skills, ~/.gemini/config/skills
                HarnessConfig {
                    id: "antigravity".to_string(),
                    name: "Google Antigravity".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 5. OpenCode (Anomaly Terminal Assistant)
                // Dual Support: Natively checks .opencode/skills, .agents/skills, and .claude/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .opencode/skills, ~/.config/opencode/skills
                HarnessConfig {
                    id: "opencode".to_string(),
                    name: "OpenCode".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 6. GitHub Copilot (VS Code Agent Mode & CLI)
                // Dual Support: Discovers .agents/skills alongside .github/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .github/skills, ~/.copilot/skills
                HarnessConfig {
                    id: "github-copilot".to_string(),
                    name: "GitHub Copilot".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 7. Windsurf (Codeium Cascade)
                // Dual Support: Cascade scans .windsurf/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .windsurf/skills, ~/.codeium/windsurf/skills
                HarnessConfig {
                    id: "windsurf".to_string(),
                    name: "Windsurf".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 8. Pi (pi-coding-agent)
                // Dual Support: Discovers .pi/skills and .agents/skills upon workspace trust.
                // Preferred standard path selected.
                // Alternative proprietary: .pi/skills, ~/.pi/agent/skills
                HarnessConfig {
                    id: "pi".to_string(),
                    name: "Pi".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 9. Cline (VS Code Extension)
                // Dual Support: Discovers .cline/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .cline/skills, ~/.cline/skills
                HarnessConfig {
                    id: "cline".to_string(),
                    name: "Cline".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },

                // 10. Qoder (Qoder IDE & CLI)
                // Proprietary: Relies on .qoder/skills and ~/.qoder/skills for stable discovery.
                // Standard .agents/skills discovery is incomplete across releases.
                HarnessConfig {
                    id: "qoder".to_string(),
                    name: "Qoder".to_string(),
                    relative_path: PathBuf::from(".qoder/skills"),
                    global_path: Some(PathBuf::from("~/.qoder/skills")),
                    is_pinned: false,
                },

                // 11. Roo Code (VS Code Extension)
                // Dual Support: Discovers .roo/skills and .agents/skills.
                // Preferred standard path selected.
                // Alternative proprietary: .roo/skills, ~/.roo/skills
                HarnessConfig {
                    id: "roo-code".to_string(),
                    name: "Roo Code".to_string(),
                    relative_path: PathBuf::from(".agents/skills"),
                    global_path: Some(PathBuf::from("~/.agents/skills")),
                    is_pinned: false,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SourceConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TargetConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub is_global: bool,
    pub is_pinned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HarnessConfig {
    pub id: String,
    pub name: String,
    pub relative_path: PathBuf,
    pub global_path: Option<PathBuf>,
    pub is_pinned: bool,
}
```
