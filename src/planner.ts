import path from 'node:path';
import { lstat, realpath } from 'node:fs/promises';
import type { Config } from './config.js';
import { resolveProjectSkillsDir } from './config.js';
import type { Skill } from './discovery.js';

export interface CopyOperation { skill: string; source: string; target: string; tools: string[]; project?: string; mode?: 'new' | 'replace' }
export interface PlanInput { config: Config; skills: Skill[]; toolIds: string[]; scope: 'global' | 'project'; projects?: string[] }

export function buildPlan(input: PlanInput): CopyOperation[] {
  const operations = new Map<string, CopyOperation>();
  const projects = input.scope === 'global' ? [undefined] : input.projects || [];
  if (input.scope === 'project' && !projects.length) throw new Error('Select at least one project');
  for (const project of projects) for (const toolId of input.toolIds) {
    const tool = input.config.tools[toolId];
    if (!tool) throw new Error(`Unknown tool: ${toolId}`);
    const targetDir = project ? resolveProjectSkillsDir(project, tool.projectSkillsDir) : path.resolve(tool.globalSkillsDir);
    for (const skill of input.skills) {
      const target = path.join(targetDir, skill.name);
      const key = process.platform === 'win32' ? target.toLowerCase() : target;
      const existing = operations.get(key);
      if (existing) { if (!existing.tools.includes(tool.name)) existing.tools.push(tool.name); continue; }
      operations.set(key, { skill: skill.name, source: path.resolve(skill.source), target, tools: [tool.name], project });
    }
  }
  return [...operations.values()];
}

export async function annotatePlan(plan: CopyOperation[]): Promise<CopyOperation[]> {
  return Promise.all(plan.map(async (operation) => {
    let mode: 'new' | 'replace' = 'new';
    try { await lstat(operation.target); mode = 'replace'; }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    return { ...operation, mode };
  }));
}

export function formatPlan(plan: CopyOperation[]): string {
  return plan.map((item) => `${item.mode === 'replace' ? 'Replace' : 'New'} ${item.source} -> ${item.target} (${item.tools.join(', ')})`).join('\n');
}
