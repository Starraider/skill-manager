# Skill manager

Skill manager copies local skill bundles into the skills directories used by AI coding tools. A bundle is a directory containing `SKILL.md`; scripts and other files in that directory travel with it.

## Installation

Install [Git](https://git-scm.com/downloads) and [Node.js 20 or newer](https://nodejs.org/en/download) first. Node.js includes npm. Check that both are available:

```sh
git --version
node --version
npm --version
```

Clone the [GitHub repository](https://github.com/Starraider/skill-manager.git), install its locked dependencies, and build the CLI:

```sh
git clone https://github.com/Starraider/skill-manager.git
cd skill-manager
npm ci
npm run build
```

Check the command, then start the interactive installer:

```sh
node dist/cli.js --help
node dist/cli.js
```

The first run creates your YAML configuration file and asks whether to add a source directory. Select a directory containing skill bundles, either directly or in nested folders. Each bundle must contain `SKILL.md`. Then select the skills, AI tools, and global or project destinations. The CLI shows the planned copies and asks for confirmation before installing them. The [configuration](#configuration) and [interactive flow](#interactive-flow) sections below explain those choices.

To run `skill-manager` from any directory, register the built CLI with npm from the cloned repository:

```sh
npm link
skill-manager --help
skill-manager
```

This step is optional. You can always run `node dist/cli.js` from the repository directory. Use `node dist/cli.js --config /path/to/config.yaml`, or `skill-manager --config /path/to/config.yaml` after linking, to use a specific configuration file. `npm run dev -- --config /path/to/config.yaml` runs the TypeScript source while developing.

To update an existing clone, run these commands from its `skill-manager` directory:

```sh
git pull
npm ci
npm run build
```

## Configuration

The first run creates `config.yaml` with empty `sources` and `projects` lists. On macOS and Linux it lives at `${XDG_CONFIG_HOME:-~/.config}/skill-manager/config.yaml`. On Windows it lives at `%APPDATA%/skill-manager/config.yaml`. `--config` overrides this location.

The new file seeds Codex, Claude Code, Antigravity, OpenCode, and Cursor. You can edit their paths. To remove a tool, delete its entry from `tools`; the CLI will not add it back. Add another tool by giving it a unique key, a display `name`, `globalSkillsDir`, and `projectSkillsDir`:

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

`sources` and `projects` contain directory paths. Paths entered through the CLI are saved as absolute paths. Relative paths written in YAML resolve from the directory containing `config.yaml`; `~/` expands to your home directory. These directories must exist and be readable. A tool's `globalSkillsDir` must be absolute or start with `~/`. Its `projectSkillsDir` must be relative and stay inside each selected project. The CLI reports the field and config file when it finds an invalid value. It does not expand other environment variables in YAML paths.

## Interactive flow

The first question asks whether to add a source directory. The CLI opens a system folder dialog to select it when available. If the dialog cannot open, enter the directory path in the terminal. The CLI then scans all saved sources, including nested directories, for `SKILL.md`. It shows each skill by name, adding its full source path only when another skill has the same name. Select one or more skills; if two have the same directory name, choose one version. Select one or more AI tools from the current YAML file, then choose global or project installation.

For a project installation, you can add a folder before selecting projects. It uses the same graphical dialog and terminal fallback as source selection. Cancelling either dialog leaves the saved paths alone. Select at least one saved project. The CLI shows every planned source and destination, marks each destination as new or replacement, and asks before copying.

Cancelling a prompt or making an empty selection stops before any skill directory is copied.

## Replacement and failures

Installing a skill replaces the entire target directory with the same name. Files that only existed in the old copy disappear. Other skill directories in the target folder stay in place. If selected tools share a skills directory, the CLI copies each selected skill there once.

Before copying, the CLI checks sources, target paths, and target parent directories. For each destination, it copies the bundle to a temporary sibling, moves any old bundle to a temporary backup, then moves the new bundle into place. If that final move fails, it restores the old bundle. One destination can succeed while another fails, so the final report lists each source and target with its result. If a destination fails, fix the reported path or permission problem and run the CLI again.

## Development checks

```sh
npm test
npm run typecheck
npm run build
node dist/cli.js --help
```
