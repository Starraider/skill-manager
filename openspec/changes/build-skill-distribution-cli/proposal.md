# Proposal

## Why

Skills kept in several source directories are cumbersome to distribute across AI tools and projects by hand. A single CLI can make the source selection, target selection, and replacement behavior repeatable while keeping tool-specific paths editable.

## What Changes

- Add an interactive CLI that can save source directories, list their skills, and select multiple skills while preventing two versions with the same skill name from entering one installation.
- Let users select one or more AI tools after selecting skills, then install to those tools globally or to one or more saved project folders.
- Add a YAML configuration file for source paths, project folders, and each AI tool's global and project skill paths. Seed known tools once when creating the configuration; thereafter, the file alone determines which tools are selectable.
- Validate configuration and filesystem paths with actionable errors, show the planned copies, and replace an existing target skill directory with the same name.
- Offer a graphical folder picker when adding a project and a terminal path-entry fallback when a dialog cannot open.

## Capabilities

### New Capabilities

- `skill-distribution`: Interactive configuration, discovery, target selection, and copying of skill directories to global or project locations.

### Modified Capabilities

None.

## Impact

This is a new TypeScript and Node.js CLI in a repository that currently contains a project blueprint and OpenSpec files but no application code. It will add a YAML configuration format, terminal prompts, filesystem copy logic, folder-selection integration, and tests. The architecture follows `terminal-installer-project-blueprint.md`; no remote service is required.
