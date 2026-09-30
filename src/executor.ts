import { access, cp, lstat, mkdir, readdir, realpath, rename, rm, stat } from 'node:fs/promises';
import { constants } from 'node:fs';
import { randomUUID } from 'node:crypto';
import path from 'node:path';
import type { CopyOperation } from './planner.js';

export interface CopyResult extends CopyOperation { ok: boolean; error?: string }
export interface FileOps { rename: typeof rename; cp: typeof cp }
const fileOps: FileOps = { rename, cp };

function contains(parent: string, child: string): boolean {
  const relative = path.relative(parent, child);
  return !relative || (relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative));
}

async function resolvedFuturePath(value: string): Promise<string> {
  let existing = value;
  for (;;) {
    try { await lstat(existing); return path.resolve(await realpath(existing), path.relative(existing, value)); }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      const parent = path.dirname(existing);
      if (parent === existing) throw error;
      existing = parent;
    }
  }
}

async function sourceCheck(operation: CopyOperation): Promise<void> {
  const source = await realpath(operation.source);
  if (!(await stat(source)).isDirectory()) throw new Error(`Source is not a directory: ${operation.source}`);
  await access(path.join(source, 'SKILL.md'), constants.R_OK);
  async function checkTree(directory: string): Promise<void> {
    await access(directory, constants.R_OK);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const item = path.join(directory, entry.name);
      if (entry.isDirectory()) await checkTree(item);
      else if (entry.isFile()) await access(item, constants.R_OK);
    }
  }
  await checkTree(source);
  if (contains(source, operation.target) || contains(operation.target, source)) throw new Error(`Source and target overlap: ${operation.source} -> ${operation.target}`);
}

export async function preflight(plan: CopyOperation[]): Promise<void> {
  for (const operation of plan) {
    try {
      await sourceCheck(operation);
      const parent = path.dirname(operation.target);
      const futureParent = await resolvedFuturePath(parent);
      const realSource = await realpath(operation.source);
      const futureTarget = path.join(futureParent, path.basename(operation.target));
      if (contains(realSource, futureTarget) || contains(futureTarget, realSource)) throw new Error('Source and resolved target overlap');
      if (operation.project) {
        const realProject = await realpath(operation.project);
        if (!contains(realProject, futureParent)) throw new Error('Target path escapes the selected project through a symlink');
      }
      await mkdir(parent, { recursive: true });
      await access(parent, constants.W_OK);
      try { const entry = await lstat(operation.target); if (!entry.isDirectory()) throw new Error('Existing target is not a directory'); }
      catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    } catch (error) {
      throw new Error(`Preflight failed for ${operation.source} -> ${operation.target}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
}

export async function executePlan(plan: CopyOperation[], ops: FileOps = fileOps): Promise<CopyResult[]> {
  const results: CopyResult[] = [];
  for (const operation of plan) {
    const parent = path.dirname(operation.target);
    const name = path.basename(operation.target);
    const suffix = randomUUID();
    const stage = path.join(parent, `.${name}.${suffix}.stage`);
    const backup = path.join(parent, `.${name}.${suffix}.backup`);
    let backedUp = false;
    try {
      await ops.cp(operation.source, stage, { recursive: true, dereference: false, force: false, errorOnExist: true });
      try { await lstat(operation.target); await ops.rename(operation.target, backup); backedUp = true; }
      catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
      try { await ops.rename(stage, operation.target); }
      catch (error) {
        if (backedUp) { await ops.rename(backup, operation.target); backedUp = false; }
        throw error;
      }
      if (backedUp) await rm(backup, { recursive: true, force: true });
      results.push({ ...operation, ok: true });
    } catch (error) {
      results.push({ ...operation, ok: false, error: error instanceof Error ? error.message : String(error) });
    } finally { await rm(stage, { recursive: true, force: true }); }
  }
  return results;
}

export function formatResults(results: CopyResult[]): string {
  return results.map((item) => `${item.ok ? 'Installed' : 'Failed'} ${item.source} -> ${item.target} (${item.tools.join(', ')})${item.error ? `: ${item.error}` : ''}`).join('\n');
}
