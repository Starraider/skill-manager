import { afterEach, describe, expect, it } from 'vitest';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { addSavedDirectory, addSavedProject, configLocation, defaultTools, loadConfig } from '../src/config.js';

const dirs: string[] = [];
async function fixture() { const root = await mkdtemp(path.join(os.tmpdir(), 'skill-config-')); dirs.push(root); return { root, file: path.join(root, 'config.yaml') }; }
afterEach(async () => { await Promise.all(dirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true }))); });

describe('configuration', () => {
  it('creates a config once with five documented defaults', async () => {
    const { file } = await fixture();
    const loaded = await loadConfig(file);
    expect(loaded.config.sources).toEqual([]);
    expect(loaded.config.projects).toEqual([]);
    expect(loaded.config.tools).toEqual(Object.fromEntries(Object.entries(defaultTools).map(([key, value]) => [key, { ...value, globalSkillsDir: path.join(os.homedir(), value.globalSkillsDir.slice(2)) }])));
    expect(configLocation(undefined, 'win32', { APPDATA: 'C:\\Users\\x\\AppData\\Roaming' } as NodeJS.ProcessEnv)).toContain('skill-manager');
  });

  it('reports malformed YAML and field-specific tool and path errors', async () => {
    const { root, file } = await fixture();
    await writeFile(file, 'sources: [\n');
    await expect(loadConfig(file)).rejects.toThrow(/malformed YAML/);
    await writeFile(file, 'sources: []\nprojects: []\ntools:\n  broken:\n    name: Broken\n    globalSkillsDir: ~/broken\n');
    await expect(loadConfig(file)).rejects.toThrow(/tools\.broken\.projectSkillsDir/);
    await writeFile(file, 'sources: [missing]\nprojects: []\ntools: {}\n');
    await expect(loadConfig(file)).rejects.toThrow(/sources\.0.*missing/);
    await writeFile(file, 'sources: []\nprojects: [missing]\ntools: {}\n');
    await expect(loadConfig(file)).rejects.toThrow(/projects\.0.*missing/);
    await writeFile(file, 'sources: []\nprojects: []\ntools:\n  escape:\n    name: Escape\n    globalSkillsDir: ~/escape\n    projectSkillsDir: ../outside\n');
    await expect(loadConfig(file)).rejects.toThrow(/tools\.escape\.projectSkillsDir/);
    await writeFile(file, 'sources: []\nprojects: []\ntools:\n  escape:\n    name: Escape\n    globalSkillsDir: relative\n    projectSkillsDir: .agents\/skills\n');
    await expect(loadConfig(file)).rejects.toThrow(/tools\.escape\.globalSkillsDir/);
    await mkdir(path.join(root, 'source'));
    await writeFile(file, 'sources: [source]\nprojects: []\ntools: {}\n');
    expect((await loadConfig(file)).config.sources).toEqual([path.join(root, 'source')]);
  });

  it('preserves comments and manual tool edits across path saves and restart', async () => {
    const { root, file } = await fixture();
    const source = path.join(root, 'source'); const project = path.join(root, 'project');
    await mkdir(source); await mkdir(project);
    await writeFile(file, '# My tool list\nsources: []\nprojects: []\ntools:\n  custom: # keep this\n    name: Custom\n    globalSkillsDir: ~/custom/skills\n    projectSkillsDir: .custom/skills\n');
    const loaded = await loadConfig(file);
    await addSavedDirectory(loaded, 'sources', source);
    await addSavedProject(loaded, 'My Project', project);
    await expect(addSavedProject(loaded, 'My Project', project)).rejects.toThrow(/name already exists/);
    const saved = await readFile(file, 'utf8');
    expect(saved).toContain('# My tool list');
    expect(saved).toContain('# keep this');
    const restarted = await loadConfig(file);
    expect(Object.keys(restarted.config.tools)).toEqual(['custom']);
    expect(restarted.config.projects).toEqual([{ name: 'My Project', path: project }]);
  });

  it('migrates saved path entries to unique names and preserves comments', async () => {
    const { root, file } = await fixture();
    const first = path.join(root, 'one', 'shared');
    const second = path.join(root, 'two', 'shared');
    await mkdir(first, { recursive: true }); await mkdir(second, { recursive: true });
    await writeFile(file, `# keep this\nsources: []\nprojects:\n  - ${first} # first project\n  - ${second}\ntools: {}\n`);
    const loaded = await loadConfig(file);
    expect(loaded.config.projects).toEqual([{ name: 'shared', path: first }, { name: 'shared (2)', path: second }]);
    const migrated = await readFile(file, 'utf8');
    expect(migrated).toContain('# keep this');
    expect(migrated).toContain('# first project');
    expect(migrated).toContain('name: shared');
    expect((await loadConfig(file)).config.projects).toEqual(loaded.config.projects);
  });

  it('validates named project fields and duplicate names', async () => {
    const { root, file } = await fixture();
    const folder = path.join(root, 'project'); await mkdir(folder);
    await writeFile(file, `sources: []\nprojects:\n  - name: Test\n    path: missing\ntools: {}\n`);
    await expect(loadConfig(file)).rejects.toThrow(/projects\.0\.path.*missing/);
    await writeFile(file, `sources: []\nprojects:\n  - name: Test\n    path: ${folder}\n  - name: test\n    path: ${folder}\ntools: {}\n`);
    await expect(loadConfig(file)).rejects.toThrow(/projects\.1\.name/);
    await writeFile(file, `sources: []\nprojects:\n  - name: ''\n    path: ${folder}\ntools: {}\n`);
    await expect(loadConfig(file)).rejects.toThrow(/projects\.0\.name/);
  });

  it('loads the YAML example in README', async () => {
    const { file } = await fixture();
    const readme = await readFile(path.resolve('README.md'), 'utf8');
    const example = readme.match(/```yaml\n([\s\S]*?)\n```/)?.[1];
    expect(example).toBeDefined();
    await writeFile(file, example!);
    expect(Object.keys((await loadConfig(file)).config.tools)).toEqual(['codex', 'custom-agent']);
  });
});
