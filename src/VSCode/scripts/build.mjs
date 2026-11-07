import { build } from 'esbuild';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const result = await build({
  absWorkingDir: root,
  entryPoints: { extension: 'src/extension.ts', 'test/host': 'test/host.ts' },
  outdir: 'out',
  outExtension: { '.js': '.cjs' },
  bundle: true,
  platform: 'node',
  target: 'node22',
  format: 'cjs',
  external: ['vscode'],
  legalComments: 'inline',
  metafile: true,
});

const dependencies = new Set();
for (const input of Object.keys(result.metafile.outputs['out/extension.cjs'].inputs)) {
  const marker = input.lastIndexOf('node_modules/');
  if (marker < 0) continue;
  const prefix = input.slice(0, marker + 'node_modules/'.length);
  const parts = input.slice(prefix.length).split('/');
  dependencies.add(prefix + parts.slice(0, parts[0].startsWith('@') ? 2 : 1).join('/'));
}
const notices = [];
for (const dependency of [...dependencies].sort()) {
  const directory = path.join(root, dependency);
  const manifest = JSON.parse(await readFile(path.join(directory, 'package.json'), 'utf8'));
  const license = (await readdir(directory)).find(name => /^licen[sc]e(\.(txt|md))?$/i.test(name));
  if (!license) throw new Error(`Missing license text for bundled package ${manifest.name}.`);
  notices.push(`${manifest.name} ${manifest.version}\n\n${await readFile(path.join(directory, license), 'utf8')}`);
}
await writeFile(path.join(root, 'out/ThirdPartyNotices.txt'), notices.join('\n\n'));
