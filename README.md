# Skill-Manager

**Skill-Manager** is a high-performance cross-platform desktop application built in Rust and [Dioxus 0.7](https://dioxuslabs.com/) designed to discover, distribute, and synchronize LLM agent skills across multiple AI coding assistants, development projects, and global environments.

---

## Table of Contents

- [What Skill-Manager Does](#what-skill-manager-does)
  - [The Problem](#the-problem)
  - [How Skill-Manager Solves It](#how-skill-manager-solves-it)
  - [Core Features & Architecture](#core-features--architecture)
  - [Key Concepts](#key-concepts)
  - [Built-In Harness Defaults](#built-in-harness-defaults)
- [How to Compile Skill-Manager](#how-to-compile-skill-manager)
  - [Prerequisites & System Dependencies](#prerequisites--system-dependencies)
  - [Compiling with Cargo (Standalone Binary)](#1-compiling-with-cargo-standalone-binary)
  - [Compiling with Dioxus CLI (Application Bundles)](#2-compiling-with-dioxus-cli-application-bundles)
  - [Running Tests Prior to Compilation](#running-tests-prior-to-compilation)
- [How to Start the Fully Compiled Program](#how-to-start-the-fully-compiled-program)
  - [Starting the Standalone Binary](#1-starting-the-standalone-binary)
  - [Starting the OS Desktop Bundle](#2-starting-the-os-desktop-bundle)
  - [Installing System-Wide](#3-installing-system-wide)
  - [First Launch & Configuration](#first-launch--configuration)
  - [Quick-Start User Workflow](#quick-start-user-workflow)
- [Project Structure](#project-structure)
- [License](#license)

---

## What Skill-Manager Does

### The Problem

Modern AI-assisted software development relies on diverse agentic coding tools—such as **OpenAI Codex**, **Anthropic Claude Code**, **Cursor**, and **Google Antigravity**. While agent skills provide a standard way to extend these assistants with domain-specific knowledge, tools, and workflows, each assistant relies on distinct filesystem conventions:

- Project-level relative directories vary widely (e.g., `.codex/skills`, `.claude/skills`, `.cursor/skills`, `.antigravity/skills`).
- Global user directories are fragmented (e.g., `~/.agents/skills`, `~/.claude/skills`, `~/.cursor/skills`, `~/.gemini/antigravity/skills`).
- Maintaining custom skills across dozens of local project repositories and global paths requires tedious, error-prone manual file copying or ad-hoc shell scripts.
- Multiple harnesses often share identical paths, causing redundant disk writes or lock contention.
- There is no central interface to inspect which skills are active in a project or remove a skill cleanly across all tools in one step.

### How Skill-Manager Solves It

Skill-Manager provides a centralized native desktop dashboard that acts as the single control plane for your agent skills:

1. **Automatic Skill Discovery**: Scans your local source repositories for valid skill packages marked by `SKILL.md` (or `skill.md`), parsing YAML frontmatter metadata (name, title, description).
2. **Multi-Harness Fan-Out Deployment**: Lets you select one or more skills and fan them out across all configured harnesses and target repositories (or system-wide global directories) in a single click.
3. **Smart Path Canonicalization & Deduplication**: Canonicalizes and deduplicates destination paths before executing copy or deletion operations, guaranteeing that shared folders receive exactly one write.
4. **Parent Directory Initialization**: Automatically creates missing directory trees (`create_dir_all`) as needed.
5. **Synchronized Multi-Harness Removal**: Uninstalling a skill from a project or global target wipes it from all associated harness subdirectories in one atomic action, leaving your original source files untouched.
6. **Resilient Non-Blocking I/O**: Filesystem operations (scanning, deep copying, deletion) run asynchronously in background threads, keeping the desktop UI fluid and responsive even during large batch deployments.
7. **Atomic, Self-Healing Configuration**: All sources, targets, and harnesses are saved atomically to `~/.config/skill-manager/config.json`. If corrupted or absent, the app gracefully recovers with safe, pre-seeded defaults.

---

### Core Features & Architecture

```text
+-----------------------------------------------------------------------------------+
|                                  Dioxus 0.7 GUI                                   |
|   [Tab 1: Sources]              [Tab 2: Targets]            [Tab 3: Harnesses]    |
|   • Source directories          • Project directories       • Harness matrix      |
|   • Skill discovery & parsing   • Installed skills view     • Relative & global   |
|   • Multi-select & Copy Modal   • Synchronized removal      • Custom harnesses    |
+-----------------------------------------------------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                                Core Engine Services                               |
|   • fs_scanner: Detects SKILL.md, parses YAML frontmatter, sanitizes names        |
|   • deployer: Fan-out copy, path deduplication, directory init, error resilience  |
|   • config_store: Atomic persistence (~/.config/skill-manager/config.json)        |
+-----------------------------------------------------------------------------------+
```

- **Native OS Dialogs**: Integrates native folder pickers via `rfd` (Rust File Dialogs) for choosing source and target paths.
- **Zero Runtime Dependencies**: The UI is styled with embedded CSS (`include_str!("../../assets/style.css")`), compiling into a single self-contained binary that uses the native OS webview (WebKit on macOS/Linux, WebView2 on Windows).
- **Graceful Error Resilience**: If copying to one harness fails (e.g., due to permissions), Skill-Manager reports the error for that harness without aborting deployment to the others.

---

### Key Concepts

| Concept | Description |
| :--- | :--- |
| **Source** | A local folder containing your authored or cloned skills. Immediate subdirectories containing a `SKILL.md` (or `skill.md`) marker are identified as valid skills. |
| **Skill** | A discrete directory containing instructions (`SKILL.md`), scripts, documentation, and assets. Frontmatter like `name`, `title`, and `description` is parsed automatically. |
| **Target** | A destination where skills are deployed. Can be any **Project Directory** (local git repo) or the built-in **Global** target. |
| **Global Target** | A pinned target that distributes skills across all harness user-level/global paths (e.g., `~/.claude/skills`, `~/.agents/skills`). |
| **Harness** | An AI coding assistant definition specifying its relative project directory (e.g., `.cursor/skills`) and an optional global directory (e.g., `~/.cursor/skills`). |

---

### Built-In Harness Defaults

Skill-Manager comes pre-configured with four major AI coding harnesses:

| Harness | Project Relative Path | Global Path |
| :--- | :--- | :--- |
| **Codex** | `.codex/skills` | `~/.agents/skills` |
| **Claude Code** | `.claude/skills` | `~/.claude/skills` |
| **Cursor** | `.cursor/skills` | `~/.cursor/skills` |
| **Antigravity** | `.antigravity/skills` | `~/.gemini/antigravity/skills` |

You can freely add, modify, or remove custom harnesses directly within the **Harnesses** tab.

---

## How to Compile Skill-Manager

Skill-Manager is written in Rust (2021 edition) and uses Dioxus 0.7 Desktop.

### Prerequisites & System Dependencies

Ensure you have the Rust toolchain (compiler, cargo, and standard library) installed. If you do not have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Verify your installation:
```bash
rustc --version # Rust 1.75 or later is recommended
cargo --version
```

#### OS-Specific Dependencies

Because Skill-Manager runs as a native desktop application using the system webview, your operating system may require specific development libraries:

- **macOS**:
  - Requires Xcode Command Line Tools:
    ```bash
    xcode-select --install
    ```

- **Linux (Debian / Ubuntu / Pop!_OS)**:
  - Requires GTK3 and WebKit2GTK development packages:
    ```bash
    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev pkg-config build-essential
    ```
  *(Note: If your distribution uses WebKit2GTK 4.0, install `libwebkit2gtk-4.0-dev` instead.)*

- **Linux (Fedora / RHEL)**:
  ```bash
  sudo dnf install webkit2gtk4.1-devel gtk3-devel openssl-devel
  ```

- **Linux (Arch Linux)**:
  ```bash
  sudo pacman -S webkit2gtk-4.1 gtk3
  ```

- **Windows**:
  - Microsoft Visual Studio C++ Build Tools (or Visual Studio Community).
  - Microsoft Edge WebView2 (pre-installed by default on Windows 10 and 11).

---

### 1. Compiling with Cargo (Standalone Binary)

The most direct way to compile Skill-Manager into an optimized, finished executable is using standard `cargo`:

```bash
# Clone or navigate to the repository
cd skill-manager

# Build optimized release binary
cargo build --release
```

#### Compilation Output:
- **macOS / Linux**: `target/release/skill-manager`
- **Windows**: `target\release\skill-manager.exe`

This standalone binary embeds all styling and application logic. It can be moved or copied anywhere on your system.

---

### 2. Compiling with Dioxus CLI (Application Bundles)

If you want native OS installers or bundles (such as a macOS `.app` / `.dmg`, a Linux `.deb` / `.AppImage`, or a Windows `.msi` / installer), use the official Dioxus CLI:

1. **Install the Dioxus CLI** (if not already installed):
   ```bash
   cargo install dioxus-cli --locked
   ```

2. **Bundle the Release Application**:
   ```bash
   dx bundle --release
   ```

#### Bundle Output Locations:
- **macOS**: `target/dx/skill-manager/release/macos/Skill-Manager.app` (and `.dmg`)
- **Linux**: `target/dx/skill-manager/release/linux/` (`.deb`, `.AppImage`)
- **Windows**: `target\dx\skill-manager\release\windows\` (installer or portable package)

---

### Running Tests Prior to Compilation

To run the automated test suite (verifying config storage, discovery scanning, frontmatter parsing, deduplication, and deployment):

```bash
cargo test
```

All integration tests run within temporary isolated sandboxes without modifying your home directory or existing configuration files.

---

## How to Start the Fully Compiled Program

### 1. Starting the Standalone Binary

Once compiled via `cargo build --release`, launch the executable directly from the command line:

#### macOS / Linux:
```bash
./target/release/skill-manager
```

#### Windows:
```powershell
.\target\release\skill-manager.exe
```

---

### 2. Starting the OS Desktop Bundle

If you bundled the application using `dx bundle --release`:

#### macOS:
Launch directly from the terminal:
```bash
open target/dx/skill-manager/release/macos/Skill-Manager.app
```
Or copy it to your personal or system Applications directory to launch via Spotlight or Launchpad:
```bash
cp -r target/dx/skill-manager/release/macos/Skill-Manager.app /Applications/
```

#### Linux:
Make the AppImage executable and launch it:
```bash
chmod +x target/dx/skill-manager/release/linux/*.AppImage
./target/dx/skill-manager/release/linux/*.AppImage
```
Or install the `.deb` package:
```bash
sudo dpkg -i target/dx/skill-manager/release/linux/*.deb
skill-manager
```

#### Windows:
Double-click the generated `.exe` or run the installer located in `target\dx\skill-manager\release\windows\`.

---

### 3. Installing System-Wide

To install the compiled binary into your user's Cargo binary directory (usually `~/.cargo/bin`, already present in `$PATH`):

```bash
cargo install --path .
```

You can now start Skill-Manager from any terminal simply by executing:
```bash
skill-manager
```

---

### First Launch & Configuration

When Skill-Manager launches for the first time, it automatically creates its configuration file at:

- **macOS / Linux**: `~/.config/skill-manager/config.json`
- **Windows**: `%APPDATA%\skill-manager\config.json`

The file is seeded with the default harness definitions (Codex, Claude Code, Cursor, Antigravity) and the pinned **Global** target. Any updates made through the user interface are saved atomically to this file.

---

### Quick-Start User Workflow

1. **Step 1: Add a Source Directory**
   - Click the **1. Sources** tab in the top navigation.
   - Click **+ Add** to add a source directory. In the dialog, assign a custom display name (independent of the filesystem path, e.g. `"TYPO3 Skill"`) and select or enter its directory path (e.g. `/Users/svenkalbhenn/Skills/typo3`).
   - You can edit or rename any source anytime by clicking **Edit** in the toolbar, the **✎** button on the card, or by double-clicking the item card.
   - Skill-Manager scans immediate subdirectories and lists all valid skills with their titles and descriptions.

2. **Step 2: Add Target Projects (Optional)**
   - Click the **2. Targets** tab.
   - Click **+ Add** to add a project destination. Assign a custom name (e.g. `"My Project"`) and choose or enter its project path (e.g. `/Users/svenkalbhenn/Projekte/kunden-projekt`).
   - You can edit or rename any target anytime by clicking **Edit**, the **✎** button, or double-clicking the item card.
   - The built-in **Global** target is always available.

3. **Step 3: Deploy Skills**
   - Return to the **Sources** tab.
   - Select the checkboxes for the skills you wish to distribute (or click **Select All**).
   - Click **Copy Skills (N)** in the toolbar.
   - In the deployment modal, check the destination target(s) (e.g., `web-app` or `Global`) and click **Deploy**.
   - Skill-Manager creates any missing harness directories, deduplicates paths, copies the skills, and confirms the deployment.

4. **Step 4: Inspect & Synchronize Removal**
   - In the **Targets** tab, select any target project to see all installed skills and the harnesses they support.
   - Click **Remove Skill** next to any installed skill to delete it cleanly across all configured harness folders for that project.

5. **Step 5: Configure Custom Harnesses**
   - Click the **3. Harnesses** tab to view or customize relative paths, global paths, or add new AI assistants.

---

## Project Structure

```text
skill-manager/
├── Cargo.toml               # Rust package definition and dependencies
├── Dioxus.toml              # Dioxus desktop bundle configuration
├── assets/
│   └── style.css            # Dark-mode desktop application styling
├── docs/
│   └── designs/             # Architecture and design specifications
├── src/
│   ├── main.rs              # Application entrypoint & desktop window initialization
│   ├── lib.rs               # Library exports
│   ├── error.rs             # Error types (AppError)
│   ├── components/          # Dioxus UI components
│   │   ├── layout.rs        # Main window layout, header, and tab navigation
│   │   ├── tab_sources.rs   # Tab 1: Source directory and skill discovery list
│   │   ├── tab_targets.rs   # Tab 2: Project targets and installed skill management
│   │   ├── tab_harnesses.rs # Tab 3: Harness matrix and configuration
│   │   ├── copy_modal.rs    # Skill deployment fan-out modal dialog
│   │   └── common.rs        # Reusable badges, buttons, and alert components
│   ├── models/              # Data structures and domain models
│   │   ├── config.rs        # AppConfig, SourceConfig, TargetConfig, HarnessConfig
│   │   ├── skill.rs         # Skill, InstalledSkill, DeploymentResult
│   │   └── state.rs         # Reactive AppState signals and ActiveTab enum
│   └── services/            # Core filesystem and persistence engines
│       ├── fs_scanner.rs    # Directory scanner and YAML frontmatter extractor
│       ├── deployer.rs      # Fan-out copy, path deduplication, and removal engine
│       └── config_store.rs  # Atomic JSON configuration persistence and tilde expansion
└── tests/                   # Automated integration test suite
    ├── common/              # Test fixtures and temporary directory builders
    ├── config_store_tests.rs
    ├── deployer_tests.rs
    └── fs_scanner_tests.rs
```

---

## License

This project is open-source. Refer to the repository license or project author for details.

