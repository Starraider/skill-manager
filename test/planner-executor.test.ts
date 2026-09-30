import { afterEach, expect, it } from 'vitest';
import { mkdtemp, mkdir, readFile, rm, writeFile, symlink } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { buildPlan, annotatePlan } from '../src/planner.js';
import { executePlan, formatResults, preflight } from '../src/executor.js';
import { cp, rename } from 'node:fs/promises';

let root = '';
afterEach(async () => { if (root) await rm(root, { recursive: true, force: true }); root = ''; });
async function setup() {
  root = await mkdtemp(path.join(os.tmpdir(), 'skill-plan-'));
  const source = path.join(root, 'source', 'review');
  await mkdir(path.join(source, 'scripts'), { recursive: true });
  await writeFile(path.join(source, 'SKILL.md'), 'new');
  await writeFile(path.join(source, 'scripts', 'run.sh'), 'run');
  return source;
}

it('plans multiple tools and projects, deduplicating shared directories', async () => {
  const source = await setup();
  const p1 = path.join(root, 'p1'); const p2 = path.join(root, 'p2'); await mkdir(p1); await mkdir(p2);
  const config = { sources: [], projects: [{ name: 'One', path: p1 }, { name: 'Two', path: p2 }], tools: {
    a: { name: 'A', globalSkillsDir: path.join(root, 'global'), projectSkillsDir: '.agents/skills' },
    b: { name: 'B', globalSkillsDir: path.join(root, 'global'), projectSkillsDir: '.agents/skills' },
  } };
  expect(buildPlan({ config, skills: [{ name: 'review', source }], toolIds: ['a', 'b'], scope: 'global' })).toHaveLength(1);
  const plan = buildPlan({ config, skills: [{ name: 'review', source }], toolIds: ['a', 'b'], scope: 'project', projects: [p1, p2] });
  expect(plan).toHaveLength(2);
  expect(plan[0].tools).toEqual(['A', 'B']);
  expect((await annotatePlan(plan)).every((item) => item.mode === 'new')).toBe(true);
  await preflight(plan);
  const results = await executePlan(plan);
  expect(results.every((item) => item.ok)).toBe(true);
  expect(await readFile(path.join(p2, '.agents/skills/review/scripts/run.sh'), 'utf8')).toBe('run');
});

it('replaces stale files, preserves unrelated skills, and restores failed replacement', async () => {
  const source = await setup();
  const target = path.join(root, 'target', 'review');
  await mkdir(target, { recursive: true });
  await writeFile(path.join(target, 'stale'), 'old');
  await mkdir(path.join(root, 'target', 'other'));
  await writeFile(path.join(root, 'target', 'other', 'SKILL.md'), 'other');
  const operation = { skill: 'review', source, target, tools: ['A'] };
  await preflight([operation]);
  const failing = await executePlan([operation], { cp, rename: (async (from: string, to: string) => {
    if (from.endsWith('.stage')) throw new Error('simulated final move failure');
    return rename(from, to);
  }) as typeof rename });
  expect(failing[0].ok).toBe(false);
  expect(await readFile(path.join(target, 'stale'), 'utf8')).toBe('old');
  const success = await executePlan(await annotatePlan([operation]));
  expect(success[0].ok).toBe(true);
  await expect(readFile(path.join(target, 'stale'))).rejects.toThrow();
  expect(await readFile(path.join(root, 'target', 'other', 'SKILL.md'), 'utf8')).toBe('other');
});

it('rejects source overlap and bad target before execution', async () => {
  const source = await setup();
  await expect(preflight([{ skill: 'review', source, target: source, tools: ['A'] }])).rejects.toThrow(/overlap/);
  const badParent = path.join(root, 'file'); await writeFile(badParent, 'x');
  await expect(preflight([{ skill: 'review', source, target: path.join(badParent, 'review'), tools: ['A'] }])).rejects.toThrow(/Preflight failed/);
});

it('reports a later destination failure without claiming it installed', async () => {
  const source = await setup();
  const first = path.join(root, 'a', 'review'); const second = path.join(root, 'b', 'review');
  const plan = [{ skill: 'review', source, target: first, tools: ['A'] }, { skill: 'review', source, target: second, tools: ['B'] }];
  await preflight(plan);
  const results = await executePlan(plan, { rename, cp: (async (from: string, to: string, options: unknown) => {
    if (to.includes(`${path.sep}b${path.sep}`)) throw new Error('simulated copy failure');
    return cp(from, to, options as Parameters<typeof cp>[2]);
  }) as typeof cp });
  expect(results.map((item) => item.ok)).toEqual([true, false]);
  expect(results[1].error).toContain('simulated copy failure');
  expect(formatResults(results)).toContain(`Failed ${source} -> ${second}`);
  expect(formatResults(results)).not.toContain(`Installed ${source} -> ${second}`);
});

it('rejects a project target that escapes through a symlink before creating a skill directory', async () => {
  const source = await setup();
  const project = path.join(root, 'project'); const outside = path.join(root, 'outside');
  await mkdir(project); await mkdir(outside);
  await symlink(outside, path.join(project, '.custom'));
  const target = path.join(project, '.custom', 'skills', 'review');
  await expect(preflight([{ skill: 'review', source, target, tools: ['A'], project }])).rejects.toThrow(/escapes/);
  await expect(readFile(path.join(outside, 'skills', 'review', 'SKILL.md'))).rejects.toThrow();
});
