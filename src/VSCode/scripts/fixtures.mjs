import { execFileSync } from 'node:child_process';
import { chmod, cp, mkdir, readFile, readdir, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { workflowFixtures } from './workflow-fixtures.mjs';

const [rootArg, compilerArg, configuration, feedArg, alternateArg] = process.argv.slice(2);
if (!rootArg || !compilerArg || !feedArg || !alternateArg || !['Debug', 'Release'].includes(configuration)) {
  throw new Error('Usage: node scripts/fixtures.mjs NEW_DIRECTORY COMPILER Debug|Release SDK_FEED OTHER_CONFIGURATION_FEED');
}
const [root, compiler, feed, alternate] = [rootArg, compilerArg, feedArg, alternateArg].map(value => path.resolve(value));
const windows = process.platform === 'win32';
const dotnet = process.env.DOTNET_HOST_PATH || 'dotnet';
const fixtures = fileURLToPath(new URL('../test/fixtures', import.meta.url));
await mkdir(root);

function run(executable, args, cwd = root) {
  execFileSync(executable, args, { cwd, timeout: 120_000, windowsHide: true, stdio: 'inherit' });
}
async function packageVersion(directory) {
  const packages = (await readdir(directory)).filter(name => /^Fpas\.NET\.Sdk\..+\.nupkg$/i.test(name));
  if (packages.length !== 1) throw new Error('Select a feed containing exactly one FPAS SDK package.');
  return packages[0].slice('Fpas.NET.Sdk.'.length, -'.nupkg'.length);
}
const version = await packageVersion(feed);
const otherVersion = await packageVersion(alternate);

async function project(directory, name, selectedFeed = feed, selectedVersion = version) {
  run(compiler, ['init', 'project', directory, '--name', name, '--sdk-feed', selectedFeed, '--sdk-version', selectedVersion]);
  const names = (await readdir(directory)).filter(entry => entry.endsWith('.fpasproj'));
  if (names.length !== 1) throw new Error('Expected one scaffolded project.');
  return path.join(directory, names[0]);
}
function restore(file) { run(compiler, ['restore', file, '--configuration', configuration]); }
function build(file) { run(compiler, ['build', file, '--configuration', configuration, '--no-restore']); }

async function makeExecutable(name, executable) {
  if (windows) return executable;
  // Remote-WSL owns the process environment. Give the test installations the same isolated SDK cache used during restore.
  const quote = value => `'${value.replaceAll("'", "'\\''")}'`;
  const variables = ['PATH', 'DOTNET_ROOT', 'DOTNET_HOST_PATH', 'DOTNET_CLI_HOME', 'NUGET_PACKAGES'];
  const environment = variables.filter(key => process.env[key]).map(key => `export ${key}=${quote(process.env[key])}`);
  const file = path.join(root, name);
  await writeFile(file, ['#!/bin/sh', ...environment, `exec ${quote(executable)} "$@"`, ''].join('\n'));
  await chmod(file, 0o755);
  return file;
}

const selectedCompiler = await makeExecutable('fpas', compiler);
const activeCompiler = windows ? selectedCompiler : path.join(root, 'host-path', 'fpas');
if (!windows) {
  await mkdir(path.dirname(activeCompiler));
  await symlink(selectedCompiler, activeCompiler);
}
const primary = path.join(root, 'Workspace λ');
const other = path.join(root, 'Second root');
const mainProject = await project(primary, 'EditorProbe');
const secondProject = await project(path.join(other, 'Application'), 'SecondProbe');
await cp(path.join(fixtures, 'main.fpas'), path.join(primary, 'main.fpas'));
await cp(path.join(fixtures, 'helpers.fpas'), path.join(primary, 'Helpers.fpas'));
const imports = path.join(primary, 'Inputs.props');
await writeFile(imports, '<Project><ItemGroup><Compile Include="Helpers.fpas" /></ItemGroup></Project>\n');
for (const mode of ['Debug', 'Release']) {
  await writeFile(path.join(primary, `${mode}.fpas`), `unit ${mode}Facts;\npublic const ${mode}Marker: int := 1;\nend unit;\n`);
}
let xml = await readFile(mainProject, 'utf8');
xml = xml.replace('</Project>', `  <PropertyGroup><EnableDefaultFpasCompileItems>false</EnableDefaultFpasCompileItems></PropertyGroup>
  <Import Project="Inputs.props" />
  <ItemGroup>
    <Compile Include="main.fpas" />
    <Compile Include="Debug.fpas" Condition="'$(Configuration)' == 'Debug'" />
    <Compile Include="Release.fpas" Condition="'$(Configuration)' == 'Release'" />
    <Reference Include="Metadata"><HintPath>refs/Metadata.dll</HintPath></Reference>
  </ItemGroup>
</Project>`);
await writeFile(mainProject, xml);

const referenceProject = path.join(root, 'metadata');
const references = path.join(primary, 'refs');
const replacements = path.join(root, 'replacement');
await mkdir(referenceProject);
await mkdir(references);
const referenceFile = path.join(referenceProject, 'Metadata.csproj');
await writeFile(referenceFile, '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><GenerateDocumentationFile>true</GenerateDocumentationFile><NoWarn>1591</NoWarn></PropertyGroup></Project>\n');
const referenceSource = path.join(referenceProject, 'Api.cs');
const methods = `
    /// <summary>Returns the generic value.</summary>
    public static T Echo<T>(T value, int repetitions = 1) => value;
    public static int Echo(int value) => value;
    private static int Hidden() => 0;
`;
await writeFile(referenceSource, `namespace Fixture; public static class Api { public static int Value() => 21; ${methods} }\n`);
run(dotnet, ['build', referenceFile, '-c', configuration, '-o', references, '--verbosity', 'quiet']);
await writeFile(referenceSource, `namespace Fixture; public static class Api { public static int Changed() => 21; ${methods} }\n`);
run(dotnet, ['build', referenceFile, '-c', configuration, '-o', replacements, '--no-restore', '--verbosity', 'quiet']);
restore(mainProject);
build(mainProject);
restore(secondProject);

const noRestore = await project(path.join(other, 'Unrestored'), 'Unrestored');
const mismatched = await project(path.join(other, 'Mismatched'), 'Mismatched', alternate, otherVersion);
restore(mismatched);
const standalone = await project(path.join(other, 'Snippet'), 'Snippet');
restore(standalone);

const linked = path.join(other, 'Linked');
await mkdir(linked);
await cp(path.join(path.dirname(secondProject), 'NuGet.Config'), path.join(linked, 'NuGet.Config'));
await writeFile(path.join(linked, 'Shared.fpas'), 'unit Shared; public function Answer(): int; begin return 42; end function; end unit;\n');
const owners = await Promise.all(['First', 'Second'].map(async name => {
  const directory = path.join(linked, name);
  await mkdir(directory);
  const file = path.join(directory, `${name}.fpasproj`);
  await writeFile(file, `<Project Sdk="Fpas.NET.Sdk/${version}"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType><EnableDefaultFpasCompileItems>false</EnableDefaultFpasCompileItems></PropertyGroup><ItemGroup><Compile Include="../Shared.fpas" /></ItemGroup></Project>\n`);
  return file;
}));
const solutionDir = path.join(linked, 'Selection');
run(compiler, ['init', 'solution', solutionDir, '--name', 'Linked', '--project', owners[0], '--project', owners[1]]);
const solution = path.join(solutionDir, (await readdir(solutionDir)).find(name => name.endsWith('.slnx')));
restore(solution);

const probe = await project(path.join(root, 'probe'), 'Probe');
await cp(path.join(fixtures, 'probe.fpas'), path.join(path.dirname(probe), 'main.fpas'));
run(dotnet, ['build', probe, '-c', configuration, '-p:UseAppHost=true', '--disable-build-servers', '--verbosity', 'quiet']);
const probeOutput = path.join(path.dirname(probe), 'bin', configuration, 'net10.0');
await writeFile(path.join(probeOutput, 'compiler.txt'), selectedCompiler);
const probeExecutable = await makeExecutable('broken-fpas', path.join(probeOutput, windows ? 'Probe.exe' : 'Probe'));
const restricted = path.join(root, 'Restricted');
const restrictedProject = await project(restricted, 'Restricted');
const workflows = await workflowFixtures({ primary, compiler, run, build, restore, version, feed, fixtures, configuration });
const refactorProject = await project(path.join(other, 'Refactoring λ'), 'RefactoringProbe');
const refactorSource = path.join(path.dirname(refactorProject), 'main.fpas');
const refactorDependency = path.join(path.dirname(refactorProject), 'Dependency.fpas');
await cp(path.join(fixtures, 'refactoring.fpas'), refactorSource);
await cp(path.join(fixtures, 'refactoring-dependency.fpas'), refactorDependency);
restore(refactorProject);

const data = { configuration, compiler: selectedCompiler, activeCompiler, mainProject, secondProject, noRestore, mismatched, standalone,
  imports, references, replacements, linked, solution, owners, probeExecutable, probeOutput, restrictedProject, root,
  refactorProject, refactorSource, refactorDependency, ...workflows };
const settings = { 'functionalPascalClr.configuration': configuration, 'functionalPascalClr.trace.server': 'verbose',
  'editor.formatOnSave': true, '[fpas]': { 'editor.defaultFormatter': 'functional-pascal.functional-pascal-clr' } };
await writeFile(path.join(root, 'phase1.code-workspace'), JSON.stringify({ folders: [{ path: primary }, { path: other }], settings }, null, 2));
await mkdir(path.join(restricted, '.vscode'));
await writeFile(path.join(restricted, '.vscode', 'settings.json'), JSON.stringify(settings));
for (const folder of [primary, restricted]) await writeFile(path.join(folder, 'editor-fixture.json'), JSON.stringify(data));
console.log(`Prepared ${configuration} extension-host fixtures.`);
