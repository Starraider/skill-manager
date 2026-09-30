import { afterEach, expect, it } from 'vitest';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { discoverSkills, duplicateSkillName } from '../src/discovery.js';

let root = '';
afterEach(async () => { if (root) await rm(root, { recursive: true, force: true }); root = ''; });
it('finds nested bundles and ignores ordinary Markdown', async () => {
  root = await mkdtemp(path.join(os.tmpdir(), 'skill-discovery-'));
  await mkdir(path.join(root, 'group', 'review', 'scripts'), { recursive: true });
  await mkdir(path.join(root, 'notes'));
  await writeFile(path.join(root, 'group', 'review', 'SKILL.md'), 'skill');
  await writeFile(path.join(root, 'group', 'review', 'scripts', 'run.sh'), 'echo hi');
  await writeFile(path.join(root, 'notes', 'README.md'), 'not a skill');
  expect(await discoverSkills([root])).toEqual([{ name: 'review', source: path.join(root, 'group', 'review') }]);
  expect(duplicateSkillName([{ name: 'review', source: '/a' }, { name: 'review', source: '/b' }])).toBe('review');
});
