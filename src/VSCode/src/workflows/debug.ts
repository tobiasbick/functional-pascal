import { stat } from 'node:fs/promises';
import path from 'node:path';
import * as vscode from 'vscode';
import { requireTrusted } from '../context';
import { Projects, workingDirectory, type WorkflowOptions } from './projects';

const debugType = 'fpas-clr';

interface Launch extends vscode.DebugConfiguration, WorkflowOptions {
  program?: string; stopAtEntry?: boolean; build?: boolean;
}

export function registerDebug(context: vscode.ExtensionContext, projects: Projects): void {
  context.subscriptions.push(vscode.debug.registerDebugConfigurationProvider(debugType, {
    provideDebugConfigurations() {
      return [{ type: debugType, name: 'Launch selected FPAS target', request: 'launch', stopAtEntry: false }];
    },
    resolveDebugConfigurationWithSubstitutedVariables(_folder, raw, token) {
      return projects.guard(async () => {
        const launch = raw as Launch;
        if (launch.request !== 'launch' || launch.noDebug || launch.console || launch.env || launch.processId !== undefined)
          throw new Error('FPAS CLR debugging supports launch with a built DLL, cwd, args and stopAtEntry.');
        if (launch.program !== undefined && (launch.build || launch.project !== undefined))
          throw new Error('Choose either an explicit DLL or a project launch. The optional build applies to projects.');
        if (launch.args !== undefined && (!Array.isArray(launch.args) || launch.args.some(item => typeof item !== 'string' || item.includes('\0'))))
          throw new Error('Debugger args must be an array of strings.');
        const selection = await projects.selection(launch);
        let program = launch.program;
        let directory = selection.cwd;
        if (program === undefined) {
          const target = await projects.target(selection, launch.project, token);
          if (launch.build) {
            const code = await projects.captured('build', launch, token ?? new vscode.CancellationTokenSource().token);
            if (code !== 0) throw new Error(`The pre-launch FPAS build failed (exit ${code}). See Functional Pascal CLR output.`);
            program = (await projects.target(selection, target.project, token)).assembly;
          } else program = target.assembly;
          directory = path.dirname(target.project);
        } else if (typeof program === 'string') directory = path.dirname(program);
        if (typeof program !== 'string' || !path.isAbsolute(program) || path.extname(program).toLowerCase() !== '.dll')
          throw new Error('Select an absolute managed DLL on this workspace host.');
        for (const file of [program, program.slice(0, -4) + '.pdb', program.slice(0, -4) + '.runtimeconfig.json'])
          if (!(await stat(file).catch(() => undefined))?.isFile())
            throw new Error(`Missing debugger input ${path.basename(file)}. Build the target with a portable PDB first.`);
        const cwd = await workingDirectory(launch.cwd, directory);
        projects.verify(selection);
        return { ...launch, program, cwd, args: launch.args ?? [], stopAtEntry: launch.stopAtEntry ?? false };
      });
    },
  }), vscode.debug.registerDebugAdapterDescriptorFactory(debugType, {
    async createDebugAdapterDescriptor(session) {
      requireTrusted();
      const selected = await projects.selection(session.configuration as Launch);
      const { executable, report } = await projects.toolchain.resolve(selected.context, false);
      const debuggerInfo = report.debugger;
      if (debuggerInfo?.status !== 'available' || !debuggerInfo.protocols.includes('dap') ||
          !debuggerInfo.shimPath || !debuggerInfo.shimSha256 || !report.dotnet.runtimes?.some(runtime =>
            runtime.name === 'Microsoft.NETCore.App' && runtime.version.startsWith('10.')))
        throw new Error('The selected FPAS installation needs its native debug shim and a .NET 10 runtime on the workspace host.');
      projects.verify(selected);
      return new vscode.DebugAdapterExecutable(executable, ['debug', '--protocol', 'dap'], { cwd: selected.cwd });
    },
  }));
}
