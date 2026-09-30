import { access, mkdir, readFile, rename, stat, writeFile, rm } from 'node:fs/promises';
import { constants } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { parseDocument, stringify, type Document } from 'yaml';
import { z } from 'zod';

const toolSchema = z.object({ name: z.string().trim().min(1), globalSkillsDir: z.string().trim().min(1), projectSkillsDir: z.string().trim().min(1) });
const configSchema = z.object({ sources: z.array(z.string().trim().min(1)), projects: z.array(z.string().trim().min(1)), tools: z.record(z.string(), toolSchema) });
export type Tool = z.infer<typeof toolSchema>;
export type Config = z.infer<typeof configSchema>;
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

function parseConfig(document: Document, configPath: string): Config {
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
  const projects = await Promise.all(raw.projects.map((value, index) => validateDirectory(value, `projects.${index}`, base)));
  const tools = Object.fromEntries(Object.entries(raw.tools).map(([key, tool]) => [key, { ...tool, globalSkillsDir: expandPath(tool.globalSkillsDir, base) }]));
  return { path: configPath, document, config: { sources, projects, tools } };
}

async function atomicWrite(configPath: string, content: string): Promise<void> {
  const temporary = path.join(path.dirname(configPath), `.${path.basename(configPath)}.${randomUUID()}.tmp`);
  try { await writeFile(temporary, content, { flag: 'wx' }); await rename(temporary, configPath); }
  catch (error) { await rm(temporary, { force: true }); throw error; }
}

export async function addSavedDirectory(loaded: LoadedConfig, kind: 'sources' | 'projects', value: string): Promise<string> {
  const resolved = await validateDirectory(value, kind, process.cwd());
  if (loaded.config[kind].includes(resolved)) return resolved;
  const sequence = loaded.document.get(kind, true);
  if (!sequence || !(sequence as { add?: unknown }).add) throw new Error(`${loaded.path}: ${kind} must be a YAML list`);
  (sequence as { add(value: string): void }).add(resolved);
  await atomicWrite(loaded.path, loaded.document.toString());
  loaded.config[kind].push(resolved);
  return resolved;
}
