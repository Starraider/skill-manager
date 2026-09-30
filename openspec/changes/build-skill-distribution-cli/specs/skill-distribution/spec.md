# Spec Delta

## Purpose

Lets users select skill bundles from configured sources and install them into configurable global or project directories for selected AI tools.

## ADDED Requirements

### Requirement: Persistent YAML configuration
The CLI SHALL keep source directories, project folders, and AI tool definitions in a user-editable YAML configuration file. Each AI tool definition SHALL contain a display name, a global skills directory, and a project skills directory relative to a selected project folder.

#### Scenario: First-run configuration
- **WHEN** the CLI starts without a configuration file
- **THEN** it creates one with an empty source and project list and preconfigured definitions for Codex, Claude Code, Antigravity, OpenCode, and Cursor

#### Scenario: User removes a preconfigured tool
- **WHEN** a user removes the Codex definition from the YAML file and runs the CLI again
- **THEN** Codex is absent from the tool selection prompt and the CLI does not recreate its definition

#### Scenario: User adds an unknown tool
- **WHEN** a user adds a valid tool definition with both skill paths to the YAML file
- **THEN** the new tool appears in the tool selection prompt without a code change

### Requirement: Configuration validation
The CLI MUST validate YAML structure and configured paths before performing any skill copy. It SHALL identify the invalid field and explain how to correct it. Only tools with valid global and project path definitions SHALL be selectable.

#### Scenario: Missing tool path
- **WHEN** a configured tool lacks its `projectSkillsDir` field
- **THEN** the CLI reports that tool and field by name, performs no skill copy, and does not present the incomplete tool as selectable

#### Scenario: Invalid source or project path
- **WHEN** a configured source or selected project path does not refer to an accessible directory
- **THEN** the CLI names the path and the problem and performs no skill copy to that path

#### Scenario: Invalid project path definition
- **WHEN** a tool's project skills path is absolute or would escape the selected project folder
- **THEN** the CLI rejects that tool definition with a field-specific error before copying

### Requirement: Source path prompt and discovery
The first interactive prompt SHALL ask whether to add another source path. A valid new source SHALL be saved to the configuration. The CLI SHALL then discover and display skill directories found under all configured source paths; a skill directory contains `SKILL.md` and may contain supporting files.

#### Scenario: Add a source directory
- **WHEN** the user chooses to add a valid source directory
- **THEN** the CLI saves it and includes its discovered skills in the current selection list

#### Scenario: Discover skills from multiple sources
- **WHEN** configured source paths contain skill directories with `SKILL.md`
- **THEN** the CLI lists the skills with their source paths and lets the user select individual skills

#### Scenario: No skills are available
- **WHEN** no configured source contains a valid skill directory
- **THEN** the CLI explains that there are no skills to select and performs no installation

### Requirement: Duplicate skill names
The CLI SHALL display same-named skills from different sources as separate entries with their source paths, but MUST allow at most one skill with a given name in a single installation.

#### Scenario: Two sources contain review
- **WHEN** two source directories each contain a skill named `review`
- **THEN** both entries appear with their respective source paths
- **AND** selecting one prevents the other from being included in that installation

### Requirement: AI tool selection and installation scope
After skill selection, the CLI SHALL allow selection of one or more AI tools defined in the YAML configuration. It SHALL then ask whether the selected skills should be installed globally. Global and project installation are mutually exclusive choices for one run.

#### Scenario: Select several tools for global installation
- **WHEN** the user selects skills, selects Codex and Cursor, and chooses global installation
- **THEN** the CLI targets each selected skill to the configured global directory of each selected tool

#### Scenario: No AI tool is selected
- **WHEN** the user selects no AI tools
- **THEN** the CLI does not copy skills and explains that a tool must be selected

### Requirement: Project folder selection
For project installation, the CLI SHALL ask whether to add a project folder, offer a graphical folder-selection dialog when available, save a valid selected folder to the configuration, and offer a terminal path-entry fallback if the dialog is unavailable or fails. It SHALL then show all saved project folders and allow selection of one or more.

#### Scenario: Add and select a project
- **WHEN** the user chooses project installation, adds a valid project folder through the dialog, and selects it
- **THEN** the CLI saves the folder and includes it in the project selection list for the current run

#### Scenario: Graphical dialog unavailable
- **WHEN** the CLI cannot open a graphical folder dialog
- **THEN** it offers terminal path entry and validates the entered directory

#### Scenario: Select several projects
- **WHEN** the user selects two saved projects and two AI tools
- **THEN** the CLI targets every selected skill to each selected tool's project skills directory within both projects

### Requirement: Skill copying and replacement
The CLI SHALL copy the entire selected skill directory, including supporting files, into each resolved target skills directory under the skill's name. It SHALL show the planned source-to-target copies before execution, replace an existing skill directory with the same name, and leave unrelated skills in the target directory untouched.

#### Scenario: Replace an existing skill
- **WHEN** a selected target already contains a skill directory with the same name
- **THEN** the CLI replaces that directory with the selected source skill directory, including removal of files that existed only in the old version

#### Scenario: Preserve unrelated skills
- **WHEN** a target contains skill directories whose names are not selected for installation
- **THEN** those directories remain unchanged

#### Scenario: Two selected tool definitions resolve to one target
- **WHEN** selected tools resolve to the same target skills directory
- **THEN** the CLI plans and performs one copy per selected skill for that directory

#### Scenario: Copy fails
- **WHEN** the CLI cannot copy a selected skill to a target
- **THEN** it reports the source, target, and failure and does not claim that target succeeded

### Requirement: Cancellation and completion reporting
The CLI SHALL stop without copying skills when the user cancels during selection, and it SHALL summarize successful and failed destinations after a copy attempt.

#### Scenario: Cancel before installation
- **WHEN** the user cancels at a selection prompt before copying starts
- **THEN** no target skill directory is modified

#### Scenario: Completed installation
- **WHEN** all planned copies succeed
- **THEN** the CLI reports which skills were installed to which selected tools and locations
