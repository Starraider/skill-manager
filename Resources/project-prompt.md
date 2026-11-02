# Skill-Manager

## Idea

The basic idea is to manage skills for LLMs in one or more folders (source), and then either distribute these skills to individual projects (targets) or install them globally. It should also be possible to remove the skills again, either from the source or globally. It should also be possible to define for which Harness/IDE these skills are to be installed.

# Technical Requirements

The application should be implemented as a desktop application, build in Rust with the GUI made with Dioxus (<https://dioxuslabs.com/learn/0.7/> ).
The source list, target list and harness/ide list should each be saved in separate JSON files in the application folder. The GUI should have a dark theme.

## Structure

The GUI should have three main tabs:

1. Source
2. Targets
3. Harnesses/IDs.

## Tab 1: Source

- Should be possible to add source folders to the list
- Should be possible to remove source folders from the list
- Buttons: Add Folder, Remove Folder

**Left Side of the tab:**

- List of folders (Name of the folder, Path of the source folder, Status icon (exists/not exists), Actions)

**Right Side of the tab:**

- List of skills in the selected folder
- Should be possible to select one or more skills by checking the checkbox in the list
- Should have a button "Copy Skills" (copy the selected skill to another target/global)

## Tab 2: Targets

- Should be possible to add one or more target folders to the list
- Should be possible to remove target folders from the list
- Buttons: Add Target, Remove Target
- Should have one default target, named "Global" (which uses the global harness path as its path)

**Left Side of the tab:**

- List of target folders (Name of the target, Path of the target folder, Status (exists/not exists), Actions)

**Right Side of the tab:**

- List of skills (checkbox, Name of the skill), Actions)
- Buttons: Remove Skill, Update Skill

## Tab 3: Harnesses/IDEs

- Should be possible to add a harness/ide to the list
- Should be possible to remove a harness/ide from the list
- Should have one default harness/ide, named "Default"
- Each harness/ide has a relative harness folder and a global folder path

**Left Side of the tab:**

- List of harness/ide (name, actions)

**Right Side of the tab:**

- relative harness folder, global folder path

## Logic

If someone has selected at least one skill and clicks the “Copy Skills” button, a pop-up should appear where they can select one or more targets from the list of targets. When they then click “Copy” in the pop-up, for each target the relative harness folder is appended to the stored target folder, and the corresponding skill folder is copied there in full. If a skill folder with the same name already exists, it should be completely overwritten along with all its contents. If the "Global" target is selected, the global folder path of the selected harness/ide is used instead of the relative harness folder.

1. Example:

Harness/IDE: "Codex"
Relative target folder: ".codex/skills"
Global folder: "/Users/svenkalbhenn/.agents/skills"

Target "My Project"
Path: "/Users/svenkalbhenn/Projects/My-Project"

The selected skill will be copied to: "/Users/svenkalbhenn/Projects/My-Project/.codex/skills/"

1. Example global:

Harness/IDE: "Codex"
Relative target folder: ".codex/skills"
Global folder: "/Users/svenkalbhenn/.agents/skills"

Target "Global"
Path: ""

The selected skill will be copied to: "/Users/svenkalbhenn/.agents/skills"
