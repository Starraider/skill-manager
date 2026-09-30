import { afterEach, expect, it } from 'vitest';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { loadConfig } from '../src/config.js';
import { runFlow, skillChoices, toolChoices, type Choice, type PromptIO } from '../src/prompts.js';
import { pickFolder, pickProjectFolder, pickerCommand } from '../src/picker.js';
import { discoverSkills } from '../src/discovery.js';

let root = '';
afterEach(async () => { if (root) await rm(root, { recursive: true, force: true }); root = ''; });
async function setup(two = false) {
  root = await mkdtemp(path.join(os.tmpdir(), 'skill-prompt-'));
  const first = path.join(root, 'source-a', 'review');
  await mkdir(first, { recursive: true }); await writeFile(path.join(first, 'SKILL.md'), 'a');
  if (two) { const second = path.join(root, 'source-b', 'review'); await mkdir(second, { recursive: true }); await writeFile(path.join(second, 'SKILL.md'), 'b'); }
  const file = path.join(root, 'config.yaml');
  await writeFile(file, `sources:\n  - ${path.join(root, 'source-a')}\n${two ? `  - ${path.join(root, 'source-b')}\n` : ''}projects: []\ntools:\n  custom:\n    name: Custom\n    globalSkillsDir: ${path.join(root, 'global')}\n    projectSkillsDir: .custom/skills\n`);
  return { file, first };
}

class Scripted implements PromptIO {
  seen: string[] = [];
  options: Choice<unknown>[][] = [];
  notes: string[] = [];
  constructor(private replies: unknown[]) {}
  private next(message: string) { this.seen.push(message); return this.replies.shift(); }
  async confirm(message: string) { return this.next(message) as boolean | null; }
  async input(message: string) { return this.next(message) as string | null; }
  async multi<T>(message: string, options: Choice<T>[]) { this.options.push(options); const reply = this.next(message); return typeof reply === 'function' ? (reply as (options: Choice<T>[]) => T[])(options) : reply as T[] | null; }
  note(message: string) { this.notes.push(message); }
}

it('shows both same-named skills and repeats selection until only one is chosen', async () => {
  const { file } = await setup(true);
  const config = await loadConfig(file);
  expect(skillChoices(await discoverSkills(config.config.sources)).map((choice) => choice.label)).toEqual([
    expect.stringContaining('source-a/review'), expect.stringContaining('source-b/review'),
  ]);
  expect(toolChoices(config).map((choice) => choice.value)).toEqual(['custom']);
  const ui = new Scripted([false, (options: Choice<unknown>[]) => options.map((item) => item.value), (options: Choice<unknown>[]) => [options[0].value], (options: Choice<unknown>[]) => [options[0].value], true, false]);
  expect(await runFlow(config, ui)).toBe('cancelled');
  expect(ui.seen).toEqual(['Add another source path?', 'Select skills', 'Select skills', 'Select AI tools', 'Install globally?', 'Install these skills?']);
  expect(ui.notes.some((note) => note.includes('Two selected skills'))).toBe(true);
});

it('shows source paths only for duplicate skill names', () => {
  const skills = [
    { name: 'review', source: '/sources/a/review' },
    { name: 'format', source: '/sources/b/format' },
    { name: 'review', source: '/sources/c/review' },
  ];
  expect(skillChoices(skills).map((choice) => choice.label)).toEqual([
    'review — /sources/a/review',
    'format',
    'review — /sources/c/review',
  ]);
});

it('exits on empty discovery, cancellation, or no selection without changing a target', async () => {
  const { file } = await setup();
  const config = await loadConfig(file);
  const cancelled = new Scripted([null]);
  expect(await runFlow(config, cancelled)).toBe('cancelled');
  const noSkills = new Scripted([false, []]);
  expect(await runFlow(config, noSkills)).toBe('nothing-to-install');
  const noTools = new Scripted([false, (options: Choice<unknown>[]) => [options[0].value], []]);
  expect(await runFlow(config, noTools)).toBe('nothing-to-install');
  await writeFile(file, 'sources: []\nprojects: []\ntools: {}\n');
  expect(await runFlow(await loadConfig(file), new Scripted([false]))).toBe('nothing-to-install');
  await writeFile(file, `sources: [${path.join(root, 'empty')}]\nprojects: []\ntools: {}\n`);
  await mkdir(path.join(root, 'empty'));
  expect(await runFlow(await loadConfig(file), new Scripted([false]))).toBe('nothing-to-install');
});

it('saves a picked project immediately and supports multiple selected projects', async () => {
  const { file, first } = await setup();
  const p1 = path.join(root, 'p1'); const p2 = path.join(root, 'p2'); await mkdir(p1); await mkdir(p2);
  const config = await loadConfig(file);
  await writeFile(file, (await readFile(file, 'utf8')).replace('projects: []', `projects:\n  - ${p1}`));
  const current = await loadConfig(file);
  const ui = new Scripted([false, (options: Choice<unknown>[]) => [options[0].value], (options: Choice<unknown>[]) => [options[0].value], false, true, (options: Choice<unknown>[]) => options.map((item) => item.value), true]);
  expect(await runFlow(current, ui, async () => ({ kind: 'selected', path: p2 }))).toBe('installed');
  expect(ui.options.at(-1)?.map((choice) => choice.value)).toEqual([p1, p2]);
  expect(await readFile(path.join(p2, '.custom', 'skills', 'review', 'SKILL.md'), 'utf8')).toBe('a');
  expect(await readFile(path.join(p1, '.custom', 'skills', 'review', 'SKILL.md'), 'utf8')).toBe('a');
  expect(first).toContain('review');
  expect(config.config.projects).toEqual([]);
});

it('picker adapters distinguish selection, cancellation, and unavailable display', async () => {
  expect(pickerCommand('linux', {})).toBeUndefined();
  expect(pickerCommand('darwin')?.file).toBe('osascript');
  expect(pickerCommand('win32')?.file).toBe('powershell.exe');
  expect(pickerCommand('darwin', {}, 'source')?.args.join(' ')).toContain('Select a source folder');
  expect(pickerCommand('linux', { DISPLAY: ':1' }, 'source')?.args).toContain('--title=Select a source folder');
  expect(await pickProjectFolder('linux', {})).toMatchObject({ kind: 'unavailable' });
  expect(await pickFolder('source', 'linux', {})).toMatchObject({ kind: 'unavailable' });
  const selected = await pickProjectFolder('linux', { DISPLAY: ':1' }, (async () => ({ stdout: '/tmp/project\n', stderr: '', exitCode: 0 })) as never);
  expect(selected).toEqual({ kind: 'selected', path: '/tmp/project' });
  const cancelled = await pickProjectFolder('linux', { DISPLAY: ':1' }, (async () => ({ stdout: '', stderr: '', exitCode: 1 })) as never);
  expect(cancelled).toEqual({ kind: 'cancelled' });
  const unavailable = await pickProjectFolder('linux', { DISPLAY: ':1' }, (async () => { throw new Error('missing zenity'); }) as never);
  expect(unavailable).toMatchObject({ kind: 'unavailable', reason: 'missing zenity' });
  const kdialog = await pickProjectFolder('linux', { DISPLAY: ':1' }, (async (file: string) => {
    if (file === 'zenity') throw new Error('missing zenity');
    return { stdout: '/tmp/kde-project\n', stderr: '', exitCode: 0 };
  }) as never);
  expect(kdialog).toEqual({ kind: 'selected', path: '/tmp/kde-project' });
});

it('falls back to terminal path entry when no picker is available', async () => {
  const { file } = await setup();
  const project = path.join(root, 'project'); await mkdir(project);
  const ui = new Scripted([false, (options: Choice<unknown>[]) => [options[0].value], (options: Choice<unknown>[]) => [options[0].value], false, true, project, (options: Choice<unknown>[]) => [options[0].value], false]);
  const result = await runFlow(await loadConfig(file), ui, async () => ({ kind: 'unavailable', reason: 'No display' }));
  expect(result).toBe('cancelled');
  expect(ui.seen).toContain('Project directory path');
  expect((await loadConfig(file)).config.projects).toEqual([project]);
});

it('does not create a target directory when the final confirmation is declined', async () => {
  const { file } = await setup();
  const ui = new Scripted([false, (options: Choice<unknown>[]) => [options[0].value], (options: Choice<unknown>[]) => [options[0].value], true, false]);
  expect(await runFlow(await loadConfig(file), ui)).toBe('cancelled');
  await expect(readFile(path.join(root, 'global', 'review', 'SKILL.md'))).rejects.toThrow();
  expect(ui.notes.some((note) => note.includes('New '))).toBe(true);
});

it('saves a newly entered source and offers its skills in the same run', async () => {
  const { file } = await setup();
  const second = path.join(root, 'source-b', 'new-skill');
  await mkdir(second, { recursive: true }); await writeFile(path.join(second, 'SKILL.md'), 'new');
  const ui = new Scripted([true, path.dirname(second), []]);
  expect(await runFlow(await loadConfig(file), ui, async () => ({ kind: 'unavailable', reason: 'No display' }))).toBe('nothing-to-install');
  expect(ui.seen).toContain('Source directory path');
  expect(ui.options[0].map((choice) => choice.label)).toEqual(['new-skill', 'review']);
  expect((await loadConfig(file)).config.sources).toContain(path.dirname(second));
});

it('adds a source selected in the graphical picker without a path prompt', async () => {
  const { file } = await setup();
  const second = path.join(root, 'source-b', 'new-skill');
  await mkdir(second, { recursive: true }); await writeFile(path.join(second, 'SKILL.md'), 'new');
  const ui = new Scripted([true, []]);
  const purposes: string[] = [];
  expect(await runFlow(await loadConfig(file), ui, async (purpose) => {
    purposes.push(purpose);
    return { kind: 'selected', path: path.dirname(second) };
  })).toBe('nothing-to-install');
  expect(purposes).toEqual(['source']);
  expect(ui.seen).not.toContain('Source directory path');
  expect(ui.options[0].map((choice) => choice.label)).toContain('new-skill');
  expect((await loadConfig(file)).config.sources).toContain(path.dirname(second));
});

it('leaves the source list unchanged when the graphical picker is cancelled', async () => {
  const { file } = await setup();
  const ui = new Scripted([true, []]);
  expect(await runFlow(await loadConfig(file), ui, async () => ({ kind: 'cancelled' }))).toBe('nothing-to-install');
  expect(ui.seen).not.toContain('Source directory path');
  expect((await loadConfig(file)).config.sources).toEqual([path.join(root, 'source-a')]);
});
