import { execa } from 'execa';

export type PickResult = { kind: 'selected'; path: string } | { kind: 'cancelled' } | { kind: 'unavailable'; reason: string };
export interface PickerCommand { file: string; args: string[] }
export type FolderPurpose = 'source' | 'project';

export function pickerCommand(platform = process.platform, env = process.env, purpose: FolderPurpose = 'project'): PickerCommand | undefined {
  const title = `Select a ${purpose} folder`;
  if (platform === 'darwin') return { file: 'osascript', args: ['-e', `POSIX path of (choose folder with prompt "${title}")`] };
  if (platform === 'win32') return { file: 'powershell.exe', args: ['-NoProfile', '-Command', `Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = "${title}"; if ($d.ShowDialog() -eq "OK") { Write-Output $d.SelectedPath }`] };
  if (platform === 'linux' && (env.DISPLAY || env.WAYLAND_DISPLAY)) return { file: 'zenity', args: ['--file-selection', '--directory', `--title=${title}`] };
  return undefined;
}

export async function pickFolder(
  purpose: FolderPurpose,
  platform = process.platform,
  env = process.env,
  execute: typeof execa = execa,
): Promise<PickResult> {
  const command = pickerCommand(platform, env, purpose);
  if (!command) return { kind: 'unavailable', reason: 'No graphical folder picker is available in this session' };
  const candidates = platform === 'linux'
    ? [command, { file: 'kdialog', args: ['--title', `Select a ${purpose} folder`, '--getexistingdirectory'] }]
    : [command];
  let reason = '';
  for (const candidate of candidates) {
    try {
      const result = await execute(candidate.file, candidate.args, { reject: false });
      const selected = result.stdout.trim();
      if (result.exitCode === 0 && selected) return { kind: 'selected', path: selected };
      if ((platform === 'darwin' && result.stderr.includes('-128')) || (platform !== 'darwin' && (result.exitCode === 1 || result.exitCode === 0))) return { kind: 'cancelled' };
      reason = result.stderr.trim() || `${candidate.file} exited with ${result.exitCode}`;
    } catch (error) { reason = error instanceof Error ? error.message : String(error); }
  }
  return { kind: 'unavailable', reason };
}

export function pickProjectFolder(platform = process.platform, env = process.env, execute: typeof execa = execa): Promise<PickResult> {
  return pickFolder('project', platform, env, execute);
}
