# Recommended Architecture for a Terminal-Based Project Installer

## Purpose

This document is a practical blueprint for building a modern, user-friendly command-line installer for a software project.

The installer is assumed to need capabilities such as:

- displaying interactive menus in the terminal
- letting users select installation options
- creating directories
- creating, copying, modifying, and deleting files
- running external commands
- running Bash scripts when necessary
- validating system requirements
- detecting existing installations
- supporting upgrades and reinstallation
- supporting both interactive and non-interactive operation
- providing useful error messages
- working reliably across operating systems where possible

The recommended architecture is inspired by the general approach used by installers such as BMAD's CLI installer, with a few refinements for a new project.

---

# 1. Recommended Overall Approach

For most developer-oriented installers, the recommended stack is:

```text
TypeScript
Node.js
@clack/prompts
Commander.js
Execa
node:fs/promises
node:path
YAML or JSON
Zod
Semver
Vitest
npm / npx
```

The guiding principle should be:

> Use TypeScript and Node.js as the orchestration layer. Use native Node APIs for filesystem operations. Use Execa for external processes. Use Bash only when a task genuinely belongs in Bash.

This approach provides a much cleaner and more maintainable architecture than implementing the entire installer as one large Bash script.

---

# 2. Recommended Tech Stack

## 2.1 TypeScript

Use **TypeScript** as the main implementation language.

Reasons:

- better maintainability than plain JavaScript
- compile-time type checking
- easier refactoring
- good IDE support
- safer configuration objects
- easier development of larger installers
- excellent ecosystem for CLI tooling

Recommended:

```text
TypeScript 5+
Node.js 20+ or 22+
```

If the installer is expected to be maintained for several years, TypeScript is strongly preferable to plain JavaScript.

---

## 2.2 Node.js

Use **Node.js** as the runtime.

Node is particularly suitable if the target audience is developers or if the project itself already uses JavaScript, TypeScript, Node.js, frontend build tooling, or npm.

Advantages:

- cross-platform
- excellent filesystem APIs
- large CLI ecosystem
- easy npm distribution
- easy execution through `npx`
- simple integration with Git, Docker, Composer, npm, pnpm, Bash, and other tools

Example:

```bash
npx my-project-installer
```

or:

```bash
npx my-project-installer install
```

---

## 2.3 @clack/prompts

Use **`@clack/prompts`** for the interactive terminal user interface.

This is the most important library for the visible installer experience.

It provides components such as:

```typescript
text()
select()
multiselect()
autocomplete()
confirm()
spinner()
intro()
outro()
```

Example:

```typescript
import * as p from "@clack/prompts";

p.intro("My Project Installer");

const environment = await p.select({
  message: "Choose environment",
  options: [
    {
      value: "development",
      label: "Development",
    },
    {
      value: "production",
      label: "Production",
    },
  ],
});

const components = await p.multiselect({
  message: "Select components",
  options: [
    {
      value: "docker",
      label: "Docker",
    },
    {
      value: "database",
      label: "Database",
    },
    {
      value: "nginx",
      label: "Nginx",
    },
  ],
});

const proceed = await p.confirm({
  message: "Start installation?",
});

p.outro("Installation finished.");
```

This can produce a terminal workflow similar to:

```text
┌  My Project Installer
│
◆  Choose environment
│  ● Development
│  ○ Production
│
◆  Select components
│  ◼ Docker
│  ◼ Database
│  ◻ Nginx
│
◆  Start installation?
│  Yes
│
◇  Installing...
│
└  Installation complete
```

### When to use @clack/core

For most installers, `@clack/prompts` is sufficient.

Use **`@clack/core`** only when you need custom prompt behavior such as:

- searchable multiselect menus
- custom keyboard interactions
- specialized rendering
- custom filtering
- highly customized selection behavior

Do not start with `@clack/core` unless needed.

---

# 3. Commander.js

Use **Commander.js** for CLI commands, arguments, and options.

It is useful when the installer supports commands such as:

```bash
my-installer install
my-installer update
my-installer uninstall
my-installer doctor
```

It is also useful for flags such as:

```bash
my-installer install --directory ./project
my-installer install --yes
my-installer install --dry-run
my-installer install --verbose
```

Example:

```typescript
import { Command } from "commander";

const program = new Command();

program
  .name("my-installer")
  .description("Installer for My Project")
  .version("1.0.0");

program
  .command("install")
  .description("Install the project")
  .option("-d, --directory <directory>", "Installation directory")
  .option("-y, --yes", "Skip confirmation prompts")
  .option("--dry-run", "Show actions without making changes")
  .option("--verbose", "Show detailed output")
  .action(async (options) => {
    await runInstallCommand(options);
  });

program.parse();
```

---

# 4. Execa

Use **Execa** for running external programs.

Examples:

- Git
- npm
- pnpm
- Composer
- Docker
- Docker Compose
- PHP
- Python
- system utilities
- Bash scripts

Example:

```typescript
import { execa } from "execa";

await execa("git", ["init"], {
  cwd: projectDir,
  stdio: "inherit",
});

await execa("npm", ["install"], {
  cwd: projectDir,
  stdio: "inherit",
});

await execa("docker", ["compose", "up", "-d"], {
  cwd: projectDir,
  stdio: "inherit",
});
```

Prefer:

```typescript
await execa("git", ["clone", repository, directory]);
```

over:

```typescript
exec(`git clone ${repository} ${directory}`);
```

The first approach keeps arguments separate and avoids many shell escaping and security problems.

---

# 5. Native Node Filesystem APIs

Use Node's native APIs for filesystem operations.

Recommended modules:

```typescript
import {
  mkdir,
  cp,
  copyFile,
  writeFile,
  readFile,
  rm,
  rename,
  access,
} from "node:fs/promises";

import path from "node:path";
```

Use Node APIs for:

- creating directories
- copying files
- copying directory trees
- deleting temporary files
- writing configuration files
- renaming files
- checking whether files exist

Example:

```typescript
await mkdir("./config", {
  recursive: true,
});

await cp(
  "./templates/config",
  "./config",
  {
    recursive: true,
  }
);

await writeFile(
  "./config/app.json",
  JSON.stringify(config, null, 2),
  "utf8"
);
```

Avoid using shell commands for ordinary filesystem operations:

```bash
mkdir
cp
mv
rm
```

when Node can perform the same operation directly.

This improves cross-platform compatibility and error handling.

---

# 6. node:path

Always use **`node:path`** for path handling.

Avoid manually constructing paths like:

```typescript
const target = ".config\\myproject\\settings.json";
```

Use:

```typescript
const target = path.join(
  projectDir,
  ".config",
  "myproject",
  "settings.json"
);
```

This works properly across Windows, macOS, and Linux.

---

# 7. Bash

Bash should be treated as a specialized execution tool, not as the main installer language.

Good use cases:

- existing Bash scripts
- Linux system administration
- package repository configuration
- systemd setup
- Nginx configuration helpers
- Unix-only environment setup
- operations that are naturally expressed using Unix tools

Example:

```typescript
await execa(
  "bash",
  ["./scripts/configure-server.sh"],
  {
    cwd: projectDir,
    stdio: "inherit",
  }
);
```

Recommended architecture:

```text
TypeScript
    │
    ├── terminal UI
    ├── configuration
    ├── validation
    ├── filesystem
    ├── installation workflow
    │
    └── Bash
          └── specialized system operations
```

Avoid putting all installation logic, menus, state management, and validation into one large `install.sh`.

---

# 8. Optional Alternative: zx

If the installer is extremely shell-command-heavy, **Google's `zx`** can be considered.

Example:

```typescript
import { $ } from "zx";

await $`git init`;
await $`npm install`;
await $`docker compose up -d`;
```

It makes shell-style scripting convenient from JavaScript or TypeScript.

However, for a structured installer application, the recommended default remains:

```text
Execa + node:fs/promises
```

rather than using shell commands for everything.

A good rule:

```text
Filesystem operation
        ↓
Node API

External executable
        ↓
Execa

Specialized shell script
        ↓
Bash
```

---

# 9. YAML or JSON Configuration

Avoid hard-coding every supported integration or installation option directly in TypeScript.

Prefer configuration-driven definitions where appropriate.

For example:

```yaml
id: claude-code
name: Claude Code

commands:
  directory: .claude/commands

agents:
  directory: .claude/agents
```

Another integration could be:

```yaml
id: cursor
name: Cursor

commands:
  directory: .cursor/commands

rules:
  directory: .cursor/rules
```

The installer can read these definitions and process them generically.

This makes adding future integrations much easier.

Instead of writing:

```typescript
if (tool === "cursor") {
  // ...
}

if (tool === "claude-code") {
  // ...
}

if (tool === "another-tool") {
  // ...
}
```

the installer can process data such as:

```typescript
{
  id: "cursor",
  commandsDirectory: ".cursor/commands",
  rulesDirectory: ".cursor/rules"
}
```

This is particularly useful when supporting:

- multiple IDEs
- AI coding tools
- plugins
- project templates
- modules
- deployment targets
- database types
- frameworks

---

# 10. Zod

Use **Zod** for validating configuration objects and user-provided data.

Example:

```typescript
import { z } from "zod";

const InstallConfigSchema = z.object({
  directory: z.string().min(1),

  environment: z.enum([
    "development",
    "production",
  ]),

  docker: z.boolean(),

  database: z.enum([
    "postgres",
    "mysql",
  ]),
});

type InstallConfig = z.infer<
  typeof InstallConfigSchema
>;
```

This helps catch invalid configuration before installation begins.

Zod is especially useful when configuration can come from:

- command-line arguments
- prompts
- YAML files
- JSON files
- environment variables
- saved installation state

---

# 11. Semver

Use **`semver`** if the installer needs to understand versions.

Typical use cases:

```typescript
semver.lt(installedVersion, currentVersion);
semver.gte(nodeVersion, minimumVersion);
```

Useful for:

- upgrade detection
- compatibility checks
- migration logic
- minimum dependency versions
- existing installation detection

---

# 12. Vitest

Use **Vitest** for automated testing.

A good installer should have tests for:

- configuration validation
- installation planning
- file operations
- version comparison
- upgrade paths
- command generation
- path handling
- existing installation detection
- dry-run behavior

Avoid relying only on manually running the installer.

---

# 13. Distribution Through npm / npx

For developer-facing projects, npm is a convenient distribution mechanism.

Example `package.json`:

```json
{
  "name": "my-project-installer",
  "version": "1.0.0",
  "type": "module",
  "bin": {
    "my-project": "./dist/cli.js"
  }
}
```

Your CLI entry file should begin with:

```javascript
#!/usr/bin/env node
```

Users can then run:

```bash
npx my-project-installer
```

or, depending on package naming:

```bash
npx my-project
```

---

# 14. Recommended Project Structure

A maintainable installer could use this structure:

```text
my-project-installer/
│
├── package.json
├── tsconfig.json
│
├── src/
│   │
│   ├── cli.ts
│   │
│   ├── commands/
│   │   ├── install.ts
│   │   ├── update.ts
│   │   ├── uninstall.ts
│   │   └── doctor.ts
│   │
│   ├── prompts/
│   │   ├── ask-directory.ts
│   │   ├── ask-environment.ts
│   │   ├── ask-components.ts
│   │   └── confirm-install.ts
│   │
│   ├── installer/
│   │   ├── installer.ts
│   │   ├── planner.ts
│   │   ├── executor.ts
│   │   ├── detector.ts
│   │   └── rollback.ts
│   │
│   ├── filesystem/
│   │   ├── files.ts
│   │   └── directories.ts
│   │
│   ├── process/
│   │   └── run-command.ts
│   │
│   ├── config/
│   │   ├── schema.ts
│   │   ├── loader.ts
│   │   └── writer.ts
│   │
│   └── integrations/
│       ├── loader.ts
│       └── definitions/
│           ├── cursor.yaml
│           ├── claude-code.yaml
│           └── other-tool.yaml
│
├── templates/
│   ├── config/
│   ├── commands/
│   ├── scripts/
│   └── files/
│
├── scripts/
│   ├── configure-server.sh
│   └── setup-services.sh
│
└── test/
    ├── installer.test.ts
    ├── planner.test.ts
    └── config.test.ts
```

---

# 15. Separate UI From Installer Logic

This is one of the most important architectural decisions.

The interactive menu should collect configuration.

It should not directly perform installation steps.

Recommended flow:

```text
CLI arguments
      │
      ├───────────────┐
      │               │
      ▼               ▼
Interactive UI    Non-interactive input
      │               │
      └───────┬───────┘
              ▼
       InstallConfig
              │
              ▼
        Validation
              │
              ▼
     Installation Plan
              │
              ▼
       Confirmation
              │
              ▼
          Executor
```

Both:

```bash
npx my-installer
```

and:

```bash
npx my-installer install \
  --database postgres \
  --docker \
  --yes
```

should eventually produce the same object:

```typescript
interface InstallConfig {
  directory: string;
  database: "postgres" | "mysql";
  docker: boolean;
  overwrite: boolean;
}
```

Then the installer runs:

```typescript
await install(config);
```

The installer core should not care whether configuration came from:

- interactive prompts
- command-line flags
- configuration file
- CI/CD environment

---

# 16. Installation Planning

A strong installer should build an **installation plan before making changes**.

Bad pattern:

```text
Ask question
→ immediately create directory

Ask next question
→ immediately install package
```

Recommended pattern:

```text
Questions
   ↓
Configuration
   ↓
Validation
   ↓
Installation Plan
   ↓
User Confirmation
   ↓
Execution
```

Example plan:

```typescript
const plan = [
  {
    type: "mkdir",
    path: "/project/config",
  },
  {
    type: "copy",
    from: "templates/app.yml",
    to: "/project/config/app.yml",
  },
  {
    type: "command",
    executable: "npm",
    args: ["install"],
  },
  {
    type: "command",
    executable: "docker",
    args: ["compose", "up", "-d"],
  },
];
```

This makes it possible to support:

```bash
my-installer install --dry-run
```

Example output:

```text
Installation plan

Create     ./config/
Create     ./config/app.yml
Execute    npm install
Execute    docker compose up -d

No changes have been made.
```

This is highly recommended for installers that modify real projects or servers.

---

# 17. Preflight Checks

Before installation, verify all required conditions.

Examples:

```text
Node version
Git installed
Docker installed
Docker daemon available
Composer installed
PHP version
required ports available
installation directory writable
required files present
internet access if required
disk space if relevant
existing installation state
```

Example:

```typescript
async function checkGit() {
  try {
    await execa("git", ["--version"]);
    return true;
  } catch {
    return false;
  }
}
```

Display useful results:

```text
System checks

✓ Node.js 22.12
✓ Git 2.47
✓ Docker 28.0
✓ Installation directory writable
✗ Port 8080 already in use
```

Do not begin installation if critical checks fail.

---

# 18. Existing Installation Detection

The installer should detect whether the project has already been installed.

Possible signals:

```text
.myproject/
.myproject/install.json
specific configuration files
generated directories
installed version metadata
```

For example:

```json
{
  "installerVersion": "1.4.0",
  "projectVersion": "3.2.0",
  "installedAt": "2026-09-19T12:00:00Z",
  "components": [
    "docker",
    "database"
  ]
}
```

This allows the installer to distinguish:

```text
fresh install
upgrade
repair
reconfiguration
uninstall
```

---

# 19. Idempotency

The installer should ideally be **idempotent**.

That means running it multiple times should not corrupt the installation.

For example:

```typescript
await mkdir(directory, {
  recursive: true,
});
```

rather than assuming the directory does not exist.

Before overwriting files:

```text
check whether file exists
check whether generated
check whether user modified it
decide whether to replace
backup if necessary
```

Avoid blindly replacing configuration files.

---

# 20. File Ownership / Generated File Tracking

Consider tracking which files were generated by the installer.

For example:

```json
{
  "generatedFiles": [
    ".myproject/config.json",
    ".github/agents/developer.md",
    ".cursor/rules/project.md"
  ]
}
```

This helps with:

- upgrades
- uninstall
- cleanup
- conflict detection
- safe regeneration

---

# 21. Backups

Before replacing an existing file, consider creating a backup.

Example:

```text
config.yaml
config.yaml.backup-20260919-152010
```

For important configuration files, backup behavior is strongly recommended.

---

# 22. Rollback

If installation consists of many steps, think about partial failure.

Example:

```text
1. Create directory       ✓
2. Write configuration    ✓
3. Install packages       ✓
4. Configure database     ✗
5. Start Docker
```

At this point the installer should not leave the user wondering what happened.

Possible strategies:

### Strategy A: Cleanup

Delete files created during the failed installation.

### Strategy B: Rollback stack

Record undo operations:

```typescript
rollback.push(() => rm(createdDirectory));
rollback.push(() => restoreBackup(configFile));
```

### Strategy C: Resume support

Store completed steps and continue later.

For many project installers, a combination of backups and clear recovery instructions is sufficient.

---

# 23. Cancellation Handling

Users may press:

```text
Ctrl+C
```

The installer should handle cancellation gracefully.

Possible behavior:

```text
Installation cancelled.

No existing files were modified.
Temporary files were removed.
```

Avoid leaving half-written files where possible.

---

# 24. Non-Interactive Mode

A professional installer should support automation.

Example:

```bash
my-installer install \
  --directory ./project \
  --environment production \
  --database postgres \
  --docker \
  --yes
```

This is useful for:

- CI/CD
- Docker images
- automated test environments
- provisioning
- remote deployment

Interactive and non-interactive modes should both feed the same internal configuration object.

---

# 25. Dry-Run Mode

Recommended flag:

```bash
--dry-run
```

Example:

```bash
my-installer install --dry-run
```

Output:

```text
Would create:
  ./config/
  ./data/

Would write:
  ./config/app.yaml

Would execute:
  npm install
  docker compose up -d

No changes were made.
```

Dry-run functionality is extremely useful for debugging and user trust.

---

# 26. Verbose Mode

Recommended:

```bash
--verbose
```

Normal output:

```text
Installing dependencies...
Done.
```

Verbose output:

```text
Running:
npm install

Working directory:
/home/user/project

Exit code:
0

stdout:
...
```

Keep normal mode friendly and concise.

Make verbose mode detailed enough for troubleshooting.

---

# 27. Logging

For more complex installers, consider writing a log file.

Example:

```text
.myproject/logs/install-2026-09-19.log
```

Include:

```text
installer version
operating system
Node version
selected options
commands executed
exit codes
errors
timestamps
```

Avoid logging secrets.

---

# 28. Secrets

Do not print sensitive values.

Examples:

```text
database passwords
API keys
access tokens
SSH keys
private credentials
```

If prompts request secrets, use masked input where possible.

Avoid putting secrets directly in command strings.

Prefer environment variables where suitable.

---

# 29. Shell Injection Safety

Do not construct commands like:

```typescript
exec(`some-command ${userInput}`);
```

Prefer:

```typescript
await execa(
  "some-command",
  [userInput]
);
```

Separating executable arguments significantly reduces quoting and injection risks.

---

# 30. Cross-Platform Support

If Windows support matters:

Use:

```text
node:fs/promises
node:path
Execa
```

Avoid depending heavily on:

```text
grep
sed
awk
cp
mv
rm
chmod
bash
```

unless your installer explicitly requires a Unix-like environment.

If Linux/macOS is the only supported environment, Bash usage is less problematic.

---

# 31. Suggested Commands

A mature installer could expose:

```bash
my-project install
my-project update
my-project uninstall
my-project doctor
my-project status
my-project config
```

### install

Performs installation.

### update

Updates generated files or project components.

### uninstall

Removes installer-managed files.

### doctor

Checks environment and diagnoses problems.

Example:

```text
My Project Doctor

✓ Node.js
✓ Git
✓ Docker
✓ Project configuration
✗ Database connection

Suggested action:
Check DATABASE_URL in .env
```

### status

Displays installation state.

### config

Displays or edits installer-managed configuration.

---

# 32. Recommended Installation Workflow

A robust installation process could be:

```text
1. Start CLI
2. Parse command-line options
3. Detect environment
4. Detect existing installation
5. Run preflight checks
6. Ask interactive questions if required
7. Build InstallConfig
8. Validate configuration
9. Build installation plan
10. Display plan
11. Ask for confirmation
12. Create backup if required
13. Execute filesystem operations
14. Execute external commands
15. Generate configuration files
16. Save installation metadata
17. Run verification checks
18. Display summary
```

---

# 33. Example User Experience

```text
┌  My Project Installer
│
◇  Checking system...
│
├  Node.js 22.12        ✓
├  Git 2.47             ✓
├  Docker 28.0          ✓
└  Directory writable   ✓
│
◆  Installation type
│  ● Development
│  ○ Production
│
◆  Select components
│  ◼ Docker
│  ◼ PostgreSQL
│  ◻ Nginx
│  ◻ SSL
│
◆  Installation directory
│  ./my-project
│
◇  Installation plan
│
├  Create ./config
├  Create ./data
├  Write ./config/app.yaml
├  Run npm install
└  Run docker compose up -d
│
◆  Continue?
│  Yes
│
◇  Creating directories...
◇  Writing configuration...
◇  Installing packages...
◇  Starting containers...
│
└  Installation complete ✓
```

---

# 34. Example Core Types

```typescript
export interface InstallConfig {
  directory: string;

  environment:
    | "development"
    | "production";

  database:
    | "postgres"
    | "mysql";

  docker: boolean;

  overwrite: boolean;

  dryRun: boolean;

  verbose: boolean;
}
```

---

# 35. Example Installer Entry Point

```typescript
async function install(
  config: InstallConfig
) {
  validateConfig(config);

  await runPreflightChecks(config);

  const plan =
    await buildInstallationPlan(config);

  if (config.dryRun) {
    printPlan(plan);
    return;
  }

  await executePlan(plan);

  await verifyInstallation(config);

  await saveInstallState(config);
}
```

---

# 36. Example Process Helper

Centralize external process execution.

```typescript
import { execa } from "execa";

export async function runCommand(
  command: string,
  args: string[],
  options: {
    cwd?: string;
    verbose?: boolean;
  } = {}
) {
  if (options.verbose) {
    console.log(
      `$ ${command} ${args.join(" ")}`
    );
  }

  return execa(command, args, {
    cwd: options.cwd,
    stdio: "inherit",
  });
}
```

Later, this helper can support:

- logging
- dry-run
- retry behavior
- command timing
- custom environment variables
- error formatting

---

# 37. Example File Helper

```typescript
import {
  mkdir,
  access,
} from "node:fs/promises";

export async function exists(
  filePath: string
): Promise<boolean> {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

export async function ensureDirectory(
  directory: string
) {
  await mkdir(directory, {
    recursive: true,
  });
}
```

Keep filesystem logic centralized where practical.

---

# 38. Example Installation Plan Type

```typescript
type InstallStep =
  | {
      type: "mkdir";
      path: string;
    }
  | {
      type: "copy";
      from: string;
      to: string;
    }
  | {
      type: "write";
      path: string;
      contents: string;
    }
  | {
      type: "command";
      executable: string;
      args: string[];
      cwd?: string;
    };
```

Then:

```typescript
const plan: InstallStep[] = [
  {
    type: "mkdir",
    path: "./config",
  },
  {
    type: "command",
    executable: "npm",
    args: ["install"],
  },
];
```

This architecture makes features such as dry-run, logging, rollback, testing, and preview much easier.

---

# 39. Recommended Dependency Set

A practical starting point:

```bash
npm install \
  commander \
  @clack/prompts \
  execa \
  zod \
  yaml \
  semver
```

Development dependencies:

```bash
npm install -D \
  typescript \
  tsx \
  vitest \
  @types/node
```

Optional:

```bash
npm install picocolors
```

for additional terminal coloring outside Clack.

---

# 40. Example package.json

```json
{
  "name": "my-project-installer",
  "version": "0.1.0",
  "type": "module",
  "bin": {
    "my-project": "./dist/cli.js"
  },
  "scripts": {
    "dev": "tsx src/cli.ts",
    "build": "tsc",
    "test": "vitest",
    "test:run": "vitest run"
  },
  "dependencies": {
    "@clack/prompts": "^1.0.0",
    "commander": "^14.0.0",
    "execa": "^9.0.0",
    "semver": "^7.0.0",
    "yaml": "^2.0.0",
    "zod": "^4.0.0"
  },
  "devDependencies": {
    "@types/node": "^24.0.0",
    "tsx": "^4.0.0",
    "typescript": "^5.0.0",
    "vitest": "^3.0.0"
  }
}
```

Use actual current compatible versions when starting the project rather than blindly copying version numbers from this example.

---

# 41. Recommended Development Roadmap

## Phase 1: Minimal installer

Implement:

```text
Commander
Clack
basic filesystem operations
Execa
one installation path
```

Goal:

```bash
npx my-installer
```

works end to end.

---

## Phase 2: Configuration model

Add:

```text
InstallConfig
Zod validation
config file support
```

Make UI and CLI flags produce the same configuration structure.

---

## Phase 3: Installation planner

Add:

```text
installation plan
dry-run
preview
```

Do not let UI components directly modify the filesystem.

---

## Phase 4: Reliability

Add:

```text
preflight checks
existing installation detection
backups
idempotency
error reporting
```

---

## Phase 5: Lifecycle

Add:

```text
update
repair
uninstall
doctor
status
```

---

## Phase 6: Automated testing

Test:

```text
new installation
existing installation
missing dependency
invalid config
failed command
upgrade
dry-run
uninstall
```

Use temporary directories for filesystem tests.

---

# 42. When Node Is the Right Choice

Choose the Node/TypeScript stack when:

- your users are developers
- Node is already required by the project
- npm is already part of the ecosystem
- users are comfortable running `npx`
- cross-platform behavior matters
- you want fast development
- you want a polished terminal menu

For this situation, the recommended stack is:

```text
TypeScript
Node.js
@clack/prompts
Commander.js
Execa
node:fs/promises
node:path
Zod
YAML
Semver
Vitest
npm / npx
```

---

# 43. When Go May Be Better

Consider **Go** instead if the installer must be distributed as a completely standalone executable.

Example:

```bash
./installer
```

without requiring:

```text
Node.js
npm
Python
Ruby
```

Go advantages:

- single compiled executable
- easy cross-compilation
- very good startup speed
- no runtime dependency
- strong standard library
- good terminal UI libraries

Go becomes particularly attractive for installers intended for non-developer end users.

However, development is generally more involved than a Node/TypeScript CLI.

For a developer-oriented installer, Node/TypeScript usually offers the best balance.

---

# 44. When Bash Alone Is Appropriate

A pure Bash installer can be reasonable when:

- Linux/macOS only
- very few options
- installation is short
- no complex configuration
- no upgrade system
- no interactive multiselect UI
- no long-term maintenance requirements

Example:

```text
check dependency
copy three files
run two commands
finish
```

Once the installer starts needing:

```text
menus
branching
configuration
multiple platforms
upgrades
validation
state management
rollback
testing
```

switching to a real programming language is advisable.

---

# 45. Summary Recommendation

For the described project requirements:

```text
✓ run Bash commands
✓ create directories
✓ create and copy files
✓ provide terminal menus
✓ let users select options
✓ potentially support upgrades
✓ remain maintainable
```

the recommended architecture is:

```text
                   TypeScript
                       │
       ┌───────────────┼────────────────┐
       │               │                │
       ▼               ▼                ▼
@clack/prompts     Commander           Zod
Terminal UI        CLI commands      Validation
       │               │                │
       └───────────────┴────────────────┘
                       │
                       ▼
                Installer Core
                       │
               Installation Plan
                       │
            ┌──────────┴──────────┐
            ▼                     ▼
     node:fs/promises           Execa
     directories/files      external commands
                                  │
                                  ▼
                          Bash when necessary
```

The central recommendation is:

> Use the BMAD-style Node CLI architecture, but keep the system deliberately layered: TypeScript for orchestration, Clack for menus, Commander for CLI arguments, Execa for external commands, native Node APIs for filesystem work, and Bash only for specialized system-level tasks.

This combination is simple enough to start quickly, while still being strong enough for a production-quality installer.

---

# 46. Final Suggested Stack

## Core

```text
TypeScript
Node.js
```

## CLI

```text
Commander.js
```

## Interactive terminal UI

```text
@clack/prompts
```

Optional advanced UI:

```text
@clack/core
```

## External command execution

```text
Execa
```

## Filesystem

```text
node:fs/promises
node:path
```

## Configuration

```text
YAML
JSON
```

## Validation

```text
Zod
```

## Version handling

```text
semver
```

## Testing

```text
Vitest
```

## Distribution

```text
npm
npx
```

## Optional

```text
picocolors
zx
```

Use `zx` only when the project is unusually shell-oriented.

---

# 47. Design Principles to Keep

When the project grows, keep these principles intact:

1. **UI only gathers input.**
2. **Configuration is represented as typed data.**
3. **Validation happens before installation starts.**
4. **Installation steps are planned before execution.**
5. **Filesystem work uses native Node APIs.**
6. **External programs are executed through Execa.**
7. **Bash is used only when it genuinely simplifies system-level work.**
8. **Interactive and non-interactive modes use the same installer core.**
9. **Installations should be idempotent where possible.**
10. **Existing user files should never be overwritten casually.**
11. **Dry-run support should be built into the architecture early.**
12. **Errors should explain what failed and what the user can do next.**
13. **Installer state should be tracked if updates or uninstall are supported.**
14. **Important files should be backed up before modification.**
15. **Automated tests should cover installation logic independently of the UI.**

Following these rules will produce an installer that remains manageable even after the project grows considerably beyond its first version.
