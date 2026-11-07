import { execFile } from 'node:child_process';
import { constants } from 'node:fs';
import { access, realpath, stat } from 'node:fs/promises';
import path from 'node:path';
import { promisify } from 'node:util';
import * as vscode from 'vscode';
import { requireTrusted, settings } from './context';

const execute = promisify(execFile);
const windows = process.platform === 'win32';

type Environment = {
  schemaVersion: 3;
  fpas: { cli: { version: string; sha256: string }; expectedSdkVersion: string };
  runtime: { framework: string; os: string; architecture: string };
  dotnet: { selectedSdk?: string; runtimes?: { name: string; version: string }[] };
  sdkCache: { status: string; installations: unknown[] };
  languageServer: { status: string; protocol: string; transport: string };
  debugger?: { status: string; protocols: string[]; shimPath?: string; shimSha256?: string };
  issues?: { code: string; message: string }[];
};

export class ToolchainResolver {
  private cached?: { key: string; checkedAt: number; report: Environment };

  async resolve(context: string, refresh: boolean): Promise<{ executable: string; report: Environment }> {
    requireTrusted();
    const configured = vscode.workspace.getConfiguration(settings).get<string>('executablePath', '').trim();
    const executable = await findExecutable(configured);
    const file = await stat(executable);
    const key = JSON.stringify([executable, file.size, file.mtimeMs, file.ctimeMs, context]);
    if (!refresh && this.cached?.key === key && Date.now() - this.cached.checkedAt < 60_000) {
      return { executable, report: this.cached.report };
    }

    const report = await inspect(executable, path.dirname(context));
    this.cached = { key, checkedAt: Date.now(), report };
    return { executable, report };
  }
}

async function findExecutable(configured: string): Promise<string> {
  if (configured && !path.isAbsolute(configured)) {
    throw new Error('Select an absolute path to the FPAS executable on this host.');
  }
  const candidates = configured ? [configured] : (process.env.PATH ?? '').split(path.delimiter)
    .map(directory => directory.replace(/^"|"$/g, ''))
    .filter(directory => path.isAbsolute(directory))
    .map(directory => path.join(directory, windows ? 'fpas.exe' : 'fpas'));
  for (const candidate of candidates) {
    if (windows && path.extname(candidate).toLowerCase() !== '.exe') continue;
    try {
      await access(candidate, constants.X_OK);
      if ((await stat(candidate)).isFile()) return await realpath(candidate);
    } catch { /* Continue with the next absolute PATH entry. */ }
  }
  throw new Error(configured ? `Cannot execute FPAS at ${configured}. Select an installed FPAS executable.`
    : 'FPAS is missing from this workspace host’s PATH. Select an installed FPAS executable.');
}

async function inspect(executable: string, cwd: string): Promise<Environment> {
  requireTrusted();
  let stdout: string;
  try {
    ({ stdout } = await execute(executable, ['env', '--json', '--timeout', '20'], {
      cwd, encoding: 'utf8', windowsHide: true, timeout: 24_000, maxBuffer: 1_100_000,
    }));
  } catch (failure) {
    // Inspection can report unrelated toolchain issues, such as an unavailable debugger.
    const output = (failure as { stdout?: unknown }).stdout;
    if (typeof output !== 'string' || output.length === 0) {
      throw new Error(`Could not inspect ${executable}. Check its runtime or select another FPAS executable.`);
    }
    stdout = output;
  }

  let report: Partial<Environment> | null;
  try { report = JSON.parse(stdout); }
  catch { throw new Error('FPAS returned invalid environment JSON. Select a compatible CLR FPAS executable.'); }
  if (report?.schemaVersion !== 3 || typeof report.fpas?.cli?.version !== 'string' ||
      !/^[0-9a-f]{64}$/i.test(report.fpas.cli.sha256) ||
      typeof report.fpas.expectedSdkVersion !== 'string' || typeof report.runtime?.framework !== 'string' ||
      report.runtime.os !== (windows ? 'windows' : 'linux') || report.runtime.architecture !== process.arch ||
      typeof report.sdkCache?.status !== 'string' || !Array.isArray(report.sdkCache.installations) ||
      report.languageServer?.status !== 'available' || report.languageServer.protocol !== '3.17' ||
      report.languageServer.transport !== 'stdio') {
    throw new Error('FPAS did not report a compatible CLR language server. Select a matching CLI and SDK.');
  }
  if (typeof report.dotnet?.selectedSdk !== 'string' || !/^10\./.test(report.dotnet.selectedSdk)) {
    throw new Error('Install a .NET 10 SDK on this workspace host and check the project’s global.json.');
  }
  return report as Environment;
}
