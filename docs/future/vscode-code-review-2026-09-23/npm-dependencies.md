# npm dependency review

Registry and advisory check: 2026-09-23. Versions below were obtained through `npm outdated` and `npm view <package>@latest`, not inferred from search snippets. Only stable releases are listed. No dependency or lockfile changes were made.

## Direct dependencies

| Package | Declared and installed | Latest stable | Assessment |
| --- | --- | --- | --- |
| [vscode-languageclient](https://registry.npmjs.org/vscode-languageclient/latest) | 10.1.0 | 10.1.1 | Patch update available; latest still declares VS Code `^1.91.0`. |
| [@types/adm-zip](https://registry.npmjs.org/@types%2fadm-zip/latest) | 0.5.8 | 0.5.8 | Current. |
| [@types/node](https://registry.npmjs.org/@types%2fnode/latest) | 24.13.3 | 26.6.2 | New major type definitions available; align with supported extension-host Node APIs rather than adopting solely because newer. |
| [@types/vscode](https://registry.npmjs.org/@types%2fvscode/latest) | 1.91.0 | 1.138.0 | New definitions available; current pin intentionally matches the declared minimum API. |
| [@vscode/test-electron](https://registry.npmjs.org/@vscode%2ftest-electron/latest) | 3.1.0 | 3.1.0 | Current; Node >=22. |
| [@vscode/vsce](https://registry.npmjs.org/@vscode%2fvsce/latest) | 3.9.2 | 4.0.0 | Stable major update; Node >=22. |
| [adm-zip](https://registry.npmjs.org/adm-zip/latest) | 0.6.0 | 0.6.1 | Patch update available with relevant advisory fixes; Node >=14. |
| [esbuild](https://registry.npmjs.org/esbuild/latest) | 0.25.12 | 0.28.2 | Stable update across minor lines in a 0.x package; Node >=18. |
| [typescript](https://registry.npmjs.org/typescript/latest) | 7.0.2 | 7.0.2 | Current; Node >=16.20. |
| [vscode-oniguruma](https://registry.npmjs.org/vscode-oniguruma/latest) | 2.0.1 | 2.0.1 | Current. |
| [vscode-textmate](https://registry.npmjs.org/vscode-textmate/latest) | 9.3.2 | 9.3.2 | Current. |

Six direct packages have newer stable versions. Exact pins mean `wanted` remains equal to the installed version in `npm outdated`; that does not mean no update exists. Prerelease tags such as vsce `4.0.1-0` and TypeScript development builds were excluded.

The [vsce 4.0.0 release](https://github.com/microsoft/vscode-vsce/releases/tag/v4.0.0) raises its Node baseline to 22 and changes dependency and file-discovery internals. The README already requires Node 22 or newer. Recheck the exact archive inventory after upgrading. The [esbuild 0.28.2 release](https://github.com/evanw/esbuild/releases/tag/v0.28.2) is stable, but availability alone does not prove compatibility across the intervening 0.x releases.

## N01, P2: Locked dependencies have audit findings

`npm audit --json` reports five affected package entries: four high and one moderate. `npm audit --omit=dev --json` reports one high entry. Counts are package entries, not distinct advisory counts or proven extension exploits.

| Installed package | npm severity | Dependency use | Follow-up |
| --- | --- | --- | --- |
| `brace-expansion@5.0.8` | High | Runtime through `vscode-languageclient -> minimatch@10.2.6`; also used by packaging | Refresh to a fixed compatible version, then rebuild the bundle. |
| `adm-zip@0.6.0` | High | Development/package verification | Update to 0.6.1. |
| `fast-uri@3.1.4` | High | vsce/secretlint/Ajv development chain | Refresh the dependency graph and rerun audit. |
| `js-yaml@4.3.0` | High | vsce/secretlint development chain | Refresh the dependency graph and rerun audit. |
| `qs@6.15.3` | Moderate | vsce/typed-rest-client development chain | Refresh the dependency graph and rerun audit. |

The [brace-expansion advisory](https://github.com/advisories/GHSA-rgw5-rvv9-x895) covers the installed 5.0.8 and identifies 5.0.9 as the fix boundary. Inspection of the generated extension bundle confirmed that brace-expansion code is included. `--no-dependencies` during VSIX packaging does not remove dependencies already bundled by esbuild. Reachability of an attacker-controlled pattern through this particular LSP configuration was not demonstrated.

The [adm-zip allocation advisory](https://github.com/advisories/GHSA-7q85-xj36-vmfc) affects versions below 0.6.1. The local verifier reads an archive, so allocation behavior is relevant; its ordinary input is the locally generated VSIX. The separately reported extraction/symlink advisory does not imply the verifier extracts archives, because it does not. See the [extraction advisory](https://github.com/advisories/GHSA-vwc7-r8mq-g2x9).

Other audit references include [fast-uri](https://github.com/advisories/GHSA-7p8r-x3mc-p8w7), [js-yaml](https://github.com/advisories/GHSA-2883-xcg3-v3hh), and [qs](https://github.com/advisories/GHSA-4mjr-xmp4-gh2g). npm reported fixes available for all five package entries. This was not verified by modifying the dependency graph during the review.

## Suggested update sequence

1. Update `vscode-languageclient` and `adm-zip` to their stable patch releases and refresh vulnerable transitive resolutions. Inspect the resulting lockfile and production audit.
2. Upgrade vsce to stable 4.0.0 and esbuild to stable 0.28.2 in a separate, reviewable dependency change. Run compilation, grammar/contracts, host tests, and VSIX inventory verification.
3. Choose the supported editor/runtime baseline before changing the two type-definition packages. New types can allow APIs unavailable in an older declared host.
4. Rebuild the shipped bundle after any runtime dependency update and verify the packaged artifact. An updated lockfile alone does not replace existing VSIX contents.

No `npm audit fix`, forced major migration, or package publication was performed.
