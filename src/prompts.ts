import * as clack from '@clack/prompts';
import { addSavedDirectory, loadConfig, validateDirectory, type LoadedConfig } from './config.js';
import { discoverSkills, duplicateSkillName, type Skill } from './discovery.js';
import { pickProjectFolder, type PickResult } from './picker.js';
import { annotatePlan, buildPlan, formatPlan } from './planner.js';
import { executePlan, formatResults, preflight } from './executor.js';

export interface Choice<T> { label: string; value: T; hint?: string }
export interface PromptIO {
  confirm(message: string): Promise<boolean | null>;
  input(message: string): Promise<string | null>;
  multi<T>(message: string, options: Choice<T>[]): Promise<T[] | null>;
  note(message: string): void;
}

export const terminalPrompts: PromptIO = {
  async confirm(message) { const result = await clack.confirm({ message }); return clack.isCancel(result) ? null : result; },
  async input(message) { const result = await clack.text({ message }); return clack.isCancel(result) ? null : result; },
  async multi<T>(message: string, options: Choice<T>[]) {
    const result = await clack.multiselect<T>({ message, options: options as never, required: false });
    return clack.isCancel(result) ? null : result;
  },
  note(message) { clack.log.info(message); },
};

export function skillChoices(skills: Skill[]): Choice<Skill>[] {
  return skills.map((skill) => ({ label: `${skill.name} — ${skill.source}`, value: skill }));
}

export function toolChoices(config: LoadedConfig): Choice<string>[] {
  return Object.entries(config.config.tools).map(([key, tool]) => ({ label: tool.name, value: key, hint: key }));
}

export type FlowResult = 'cancelled' | 'nothing-to-install' | 'installed' | 'failed';

export async function runFlow(config: LoadedConfig, io: PromptIO = terminalPrompts, picker: () => Promise<PickResult> = pickProjectFolder): Promise<FlowResult> {
  const addSource = await io.confirm('Add another source path?');
  if (addSource === null) return 'cancelled';
  if (addSource) {
    const entered = await io.input('Source directory path');
    if (entered === null) return 'cancelled';
    await addSavedDirectory(config, 'sources', entered);
  }
  if (!config.config.sources.length) { io.note('No source directories are configured.'); return 'nothing-to-install'; }
  const discovered = await discoverSkills(config.config.sources);
  if (!discovered.length) { io.note('No skill directories containing SKILL.md were found.'); return 'nothing-to-install'; }
  let skills: Skill[];
  for (;;) {
    const picked = await io.multi('Select skills', skillChoices(discovered));
    if (picked === null) return 'cancelled';
    if (!picked.length) { io.note('No skills selected.'); return 'nothing-to-install'; }
    const duplicate = duplicateSkillName(picked);
    if (!duplicate) { skills = picked; break; }
    io.note(`Two selected skills are named ${duplicate}. Choose one source for that name.`);
  }
  const availableTools = toolChoices(config);
  if (!availableTools.length) { io.note('No AI tools are configured.'); return 'nothing-to-install'; }
  const tools = await io.multi('Select AI tools', availableTools);
  if (tools === null) return 'cancelled';
  if (!tools.length) { io.note('Select at least one AI tool.'); return 'nothing-to-install'; }
  const global = await io.confirm('Install globally?');
  if (global === null) return 'cancelled';
  let projects: string[] | undefined;
  if (!global) {
    const addProject = await io.confirm('Add a project folder?');
    if (addProject === null) return 'cancelled';
    if (addProject) {
      const picked = await picker();
      let selected: string | undefined;
      if (picked.kind === 'selected') {
        try { selected = await validateDirectory(picked.path, 'project'); }
        catch (error) { io.note(`Folder picker returned an invalid directory: ${(error as Error).message}`); }
      }
      if (picked.kind === 'unavailable' || (picked.kind === 'selected' && !selected)) {
        io.note(picked.kind === 'unavailable' ? `${picked.reason}. Enter a path in the terminal.` : 'Enter a valid project path in the terminal.');
        const entered = await io.input('Project directory path');
        if (entered === null) return 'cancelled';
        selected = entered;
      }
      if (selected) await addSavedDirectory(config, 'projects', selected);
    }
    if (!config.config.projects.length) { io.note('No project folders are configured.'); return 'nothing-to-install'; }
    const pickedProjects = await io.multi('Select project folders', config.config.projects.map((project) => ({ label: project, value: project })));
    if (pickedProjects === null) return 'cancelled';
    if (!pickedProjects.length) { io.note('Select at least one project folder.'); return 'nothing-to-install'; }
    projects = pickedProjects;
  }
  const plan = await annotatePlan(buildPlan({ config: config.config, skills, toolIds: tools, scope: global ? 'global' : 'project', projects }));
  io.note(`Installation plan:\n${formatPlan(plan)}`);
  const proceed = await io.confirm('Install these skills?');
  if (proceed === null || !proceed) return 'cancelled';
  await preflight(plan);
  const results = await executePlan(plan);
  io.note(formatResults(results));
  return results.some((result) => !result.ok) ? 'failed' : 'installed';
}

export async function run(configPath?: string): Promise<void> {
  try {
    const loaded = await loadConfig(configPath);
    clack.intro('Skill manager');
    const result = await runFlow(loaded);
    clack.outro(result === 'installed' ? 'Installation complete.' : result === 'failed' ? 'Some destinations failed.' : 'No skills installed.');
    if (result === 'failed') process.exitCode = 1;
  } catch (error) {
    clack.log.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
