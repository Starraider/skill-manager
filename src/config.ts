import { access, mkdir, readFile, rename, stat, writeFile, rm } from 'node:fs/promises';
import { constants } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { isSeq, parseDocument, stringify, type Document } from 'yaml';
import { z } from 'zod';

const toolSchema = z.object({ name: z.string().trim().min(1), globalSkillsDir: z.string().trim().min(1), projectSkillsDir: z.string().trim().min(1) });
const projectSchema = z.object({ name: z.string().trim().min(1), path: z.string().trim().min(1) });
const configSchema = z.object({ sources: z.array(z.string().trim().min(1)), projects: z.array(z.union([projectSchema, z.string().trim().min(1)])), tools: z.record(z.string(), toolSchema) });
export type Tool = z.infer<typeof toolSchema>;
export type Project = z.infer<typeof projectSchema>;
type RawConfig = z.infer<typeof configSchema>;
export interface Config { sources: string[]; projects: Project[]; tools: Record<string, Tool> }
export interface LoadedConfig { path: string; document: Document; config: Config }

export const defaultTools: Record<string, Tool> = {
  codex: { name: 'Codex', globalSkillsDir: '~/.codex/skills', projectSkillsDir: '.codex/skills' },
  'claude-code': { name: 'Claude Code', globalSkillsDir: '~/.claude/skills', projectSkillsDir: '.claude/skills' },
  antigravity: { name: 'Antigravity', globalSkillsDir: '~/.gemini/config/skills', projectSkillsDir: '.agents/skills' },
  opencode: { name: 'OpenCode', globalSkillsDir: '~/.config/opencode/skills', projectSkillsDir: '.opencode/skills' },
  cursor: { name: 'Cursor', globalSkillsDir: '~/.cursor/skills', projectSkillsDir: '.cursor/skills' },
};

export function configLocation(override?: string, platform = process.platform, env = process.env): string {
  if (override) return path.resolve(override);
  if (platform === 'win32') return path.join(env.APPDATA || path.join(os.homedir(), 'AppData', 'Roaming'), 'skill-manager', 'config.yaml');
  return path.join(env.XDG_CONFIG_HOME || path.join(os.homedir(), '.config'), 'skill-manager', 'config.yaml');
}

export function expandPath(value: string, base: string): string {
  if (value === '~') return os.homedir();
  if (value.startsWith('~/') || value.startsWith('~\\')) return path.resolve(os.homedir(), value.slice(2));
  return path.resolve(base, value);
}

export function resolveProjectSkillsDir(project: string, relative: string): string {
  const normalized = relative.replaceAll('\\', '/');
  if (!normalized || normalized === '.' || path.isAbsolute(relative) || path.win32.isAbsolute(relative) || normalized.startsWith('~/')) throw new Error('projectSkillsDir must be a non-empty relative path inside the project');
  const destination = path.resolve(project, normalized);
  const offset = path.relative(project, destination);
  if (!offset || offset === '..' || offset.startsWith(`..${path.sep}`) || path.isAbsolute(offset)) throw new Error('projectSkillsDir must stay inside the project');
  return destination;
}

async function directory(value: string, field: string): Promise<void> {
  try { if (!(await stat(value)).isDirectory()) throw new Error('is not a directory'); await access(value, constants.R_OK); }
  catch (error) { throw new Error(`${field} (${value}) must be an accessible directory: ${error instanceof Error ? error.message : String(error)}`); }
}

export async function validateDirectory(value: string, field: string, base = process.cwd()): Promise<string> {
  const resolved = expandPath(value, base);
  await directory(resolved, field);
  return resolved;
}

function parseConfig(document: Document, configPath: string): RawConfig {
  if (document.errors.length) throw new Error(`${configPath}: malformed YAML: ${document.errors[0].message}`);
  const result = configSchema.safeParse(document.toJS());
  if (!result.success) { const issue = result.error.issues[0]; throw new Error(`${configPath}: ${issue.path.join('.')} ${issue.message}. Edit this field in the YAML file.`); }
  return result.data;
}

export async function loadConfig(override?: string): Promise<LoadedConfig> {
  const configPath = configLocation(override);
  await mkdir(path.dirname(configPath), { recursive: true });
  try { await access(configPath); }
  catch { try { await writeFile(configPath, stringify({ sources: [], projects: [], tools: defaultTools }), { flag: 'wx' }); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error; } }
  let document: Document;
  try { document = parseDocument(await readFile(configPath, 'utf8')); }
  catch (error) { throw new Error(`${configPath}: cannot read configuration: ${error instanceof Error ? error.message : String(error)}`); }
  const raw = parseConfig(document, configPath);
  for (const [key, tool] of Object.entries(raw.tools)) {
    if (!path.isAbsolute(tool.globalSkillsDir) && !path.win32.isAbsolute(tool.globalSkillsDir) && !tool.globalSkillsDir.startsWith('~/') && !tool.globalSkillsDir.startsWith('~\\')) throw new Error(`${configPath}: tools.${key}.globalSkillsDir must be absolute or start with ~/`);
    try { resolveProjectSkillsDir(process.cwd(), tool.projectSkillsDir); }
    catch (error) { throw new Error(`${configPath}: tools.${key}.projectSkillsDir: ${(error as Error).message}`); }
  }
  const base = path.dirname(configPath);
  const sources = await Promise.all(raw.sources.map((value, index) => validateDirectory(value, `sources.${index}`, base)));
  const namedProjects = new Set<string>();
  for (const [index, entry] of raw.projects.entries()) {
    if (typeof entry === 'string') continue;
    const key = entry.name.toLocaleLowerCase();
    if (namedProjects.has(key)) throw new Error(`${configPath}: projects.${index}.name must be unique. Rename this project in the YAML file.`);
    namedProjects.add(key);
  }
  const projects: Project[] = [];
  for (const [index, entry] of raw.projects.entries()) {
    const oldFormat = typeof entry === 'string';
    const projectPath = await validateDirectory(oldFormat ? entry : entry.path, oldFormat ? `projects.${index}` : `projects.${index}.path`, base);
    let name = oldFormat ? path.basename(projectPath) || projectPath : entry.name;
    if (oldFormat) {
      const stem = name;
      let suffix = 2;
      while (namedProjects.has(name.toLocaleLowerCase())) name = `${stem} (${suffix++})`;
      namedProjects.add(name.toLocaleLowerCase());
    }
    projects.push({ name, path: projectPath });
  }
  const tools = Object.fromEntries(Object.entries(raw.tools).map(([key, tool]) => [key, { ...tool, globalSkillsDir: expandPath(tool.globalSkillsDir, base) }]));
  if (raw.projects.some((entry) => typeof entry === 'string')) {
    const sequence = document.get('projects', true);
    if (!isSeq(sequence)) throw new Error(`${configPath}: projects must be a YAML list`);
    raw.projects.forEach((entry, index) => {
      if (typeof entry !== 'string') return;
      const old = sequence.items[index] as { comment?: string; commentBefore?: string } | undefined;
      const replacement = document.createNode({ name: projects[index].name, path: entry });
      replacement.comment = old?.comment;
      replacement.commentBefore = old?.commentBefore;
      sequence.items[index] = replacement;
    });
    await atomicWrite(configPath, document.toString());
  }
  return { path: configPath, document, config: { sources, projects, tools } };
}

async function atomicWrite(configPath: string, content: string): Promise<void> {
  const temporary = path.join(path.dirname(configPath), `.${path.basename(configPath)}.${randomUUID()}.tmp`);
  try { await writeFile(temporary, content, { flag: 'wx' }); await rename(temporary, configPath); }
  catch (error) { await rm(temporary, { force: true }); throw error; }
}

export async function addSavedDirectory(loaded: LoadedConfig, kind: 'sources', value: string): Promise<string> {
  const resolved = await validateDirectory(value, kind, process.cwd());
  if (loaded.config.sources.includes(resolved)) return resolved;
  const sequence = loaded.document.get(kind, true);
  if (!isSeq(sequence)) throw new Error(`${loaded.path}: ${kind} must be a YAML list`);
  sequence.add(resolved);
  await atomicWrite(loaded.path, loaded.document.toString());
  loaded.config.sources.push(resolved);
  return resolved;
}

export async function addSavedProject(loaded: LoadedConfig, name: string, value: string): Promise<Project> {
  const trimmedName = name.trim();
  if (!trimmedName) throw new Error('Project name cannot be empty');
  if (loaded.config.projects.some((project) => project.name.toLocaleLowerCase() === trimmedName.toLocaleLowerCase())) throw new Error(`Project name already exists: ${trimmedName}`);
  const resolved = await validateDirectory(value, 'project', process.cwd());
  const existing = loaded.config.projects.find((project) => project.path === resolved);
  if (existing) throw new Error(`Project folder is already saved as ${existing.name}: ${resolved}`);
  const sequence = loaded.document.get('projects', true);
  if (!isSeq(sequence)) throw new Error(`${loaded.path}: projects must be a YAML list`);
  sequence.add({ name: trimmedName, path: resolved });
  await atomicWrite(loaded.path, loaded.document.toString());
  const project = { name: trimmedName, path: resolved };
  loaded.config.projects.push(project);
  return project;
}
