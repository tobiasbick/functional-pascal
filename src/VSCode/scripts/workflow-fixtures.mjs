import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

export async function workflowFixtures({ primary, compiler, run, build, restore, version, feed, fixtures, configuration }) {
  const mixed = path.join(primary, 'Workflows');
  const engine = path.join(mixed, 'Engine');
  const host = path.join(mixed, 'Host');
  const engineProject = path.join(engine, 'Engine.fpasproj');
  const hostProject = path.join(host, 'Host.csproj');
  run(compiler, ['init', 'library', engine, '--name', 'Engine', '--sdk-version', version, '--sdk-feed', feed]);
  await cp(path.join(fixtures, 'workflow-engine.fpas'), path.join(engine, 'Engine.fpas'));
  await mkdir(host);
  await writeFile(hostProject, `<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup>
  <ItemGroup><ProjectReference Include="../Engine/Engine.fpasproj" /></ItemGroup>
</Project>\n`);
  await cp(path.join(fixtures, 'workflow-host.cs'), path.join(host, 'Program.cs'));
  const solutionDir = path.join(mixed, 'App');
  run(compiler, ['init', 'solution', solutionDir, '--name', 'Workflow', '--project', engineProject, '--project', hostProject]);
  const workflowSolution = path.join(solutionDir, 'Workflow.slnx');
  const opposite = configuration === 'Debug' ? 'Release' : 'Debug';
  const solution = await readFile(workflowSolution, 'utf8');
  await writeFile(workflowSolution, solution.replace(/(<Project Path="[^"]*Host\.csproj")\s*\/>/,
    `$1><BuildType Solution="${configuration}|*" Project="${opposite}" /></Project>`));
  restore(workflowSolution);
  build(workflowSolution);
  const testRoot = path.join(mixed, 'Testing');
  const v = path.join(testRoot, 'Vstest');
  const m = path.join(testRoot, 'Native');
  run(compiler, ['init', 'test', v, '--name', 'Vstest', '--sdk-version', version, '--sdk-feed', feed]);
  run(compiler, ['init', 'test', m, '--name', 'Native', '--framework', 'mstest', '--runner', 'mtp', '--sdk-version', version, '--sdk-feed', feed]);
  await cp(path.join(fixtures, 'workflow-vstest.fpas'), path.join(v, 'Vstest.fpas'));
  await cp(path.join(fixtures, 'workflow-mtp.fpas'), path.join(m, 'Native.fpas'));
  const vstestProject = path.join(v, 'Vstest.fpasproj');
  const mtpProject = path.join(m, 'Native.fpasproj');
  const testSolutionDir = path.join(testRoot, 'Suite');
  run(compiler, ['init', 'solution', testSolutionDir, '--name', 'Suite', '--project', vstestProject, '--project', mtpProject]);
  const testSolution = path.join(testSolutionDir, 'Suite.slnx');
  restore(testSolution);
  build(testSolution);
  return { workflowSolution, engineProject, hostProject, hostConfiguration: opposite, testSolution, vstestProject, mtpProject };
}
