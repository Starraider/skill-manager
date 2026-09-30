import { readdir, stat } from 'node:fs/promises';
import path from 'node:path';

export interface Skill { name: string; source: string }

export async function discoverSkills(roots: string[]): Promise<Skill[]> {
  const found: Skill[] = [];
  const visited = new Set<string>();
  async function visit(directory: string): Promise<void> {
    const resolved = path.resolve(directory);
    if (visited.has(resolved)) return;
    visited.add(resolved);
    const entries = await readdir(resolved, { withFileTypes: true });
    if (entries.some((entry) => entry.name === 'SKILL.md' && entry.isFile())) {
      found.push({ name: path.basename(resolved), source: resolved });
    }
    for (const entry of entries) if (entry.isDirectory()) await visit(path.join(resolved, entry.name));
  }
  for (const root of roots) { if (!(await stat(root)).isDirectory()) throw new Error(`Source is not a directory: ${root}`); await visit(root); }
  return found.sort((a, b) => a.name.localeCompare(b.name) || a.source.localeCompare(b.source));
}

export function duplicateSkillName(skills: Skill[]): string | undefined {
  const seen = new Set<string>();
  for (const skill of skills) { if (seen.has(skill.name)) return skill.name; seen.add(skill.name); }
  return undefined;
}
