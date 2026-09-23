# Documentation and online API comparison

Official documentation was checked on 2026-09-23. This report distinguishes actual mismatches from supported design choices and missing verification.

## API and protocol comparison

| Area | Implementation | Comparison |
| --- | --- | --- |
| Testing requests | `src/workflow/testing.ts` | Ignores `exclude`; F03 contradicts the [Testing API run-handler contract](https://code.visualstudio.com/api/extension-guides/testing#running-tests). |
| Test identity | Discovery uses source paths; execution/report mapping uses basenames | F02 prevents correct status mapping and precise selection for nested projects. This is a local CLI/extension contract mismatch. |
| Mouse input | `src/debugger/terminal/input.ts:186` | F06 contradicts [xterm mouse tracking](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking). |
| Project manifests | `src/debugger/projectTarget.ts` | F09 accepts only a partial string grammar compared with [TOML strings](https://toml.io/en/v1.0.0#string). |
| Workspace trust | No explicit trust capability; test runner disables trust | VS Code defaults to disabling extensions without support declarations in Restricted Mode. Absence of an `isTrusted` check alone does not establish unsafe activation. See the [Workspace Trust guide](https://code.visualstudio.com/api/extension-guides/workspace-trust#what-if-i-dont-make-changes-to-my-extension). |
| Remote execution | `extensionKind: ["ui"]`, plus explicit remote rejection in `cliPath.ts` | Local-only operation is deliberate. The [remote extension guide](https://code.visualstudio.com/api/advanced-topics/remote-extensions) distinguishes host location from connection to a remote workspace; `remoteName` describes the latter. The current diagnostic says remote extension host even when the UI extension runs locally. |
| Compatibility | Manifest and VS Code types target 1.91.0; host tests pin 1.137.0 | The [extension manifest contract](https://code.visualstudio.com/api/references/extension-manifest) makes `engines.vscode` a compatibility declaration. Testing only 1.137.0 does not verify the minimum supported version. No actual 1.91.0 failure is claimed. |
| External terminal shutdown | Writes output, then forces process exit | The [Node process documentation](https://nodejs.org/api/process.html#processexitcode) warns about unfinished output on forced exit. P03 records the unmeasured portability risk. |

## Local README claims that need correction or implementation work

`editors/vscode/README.md:241` promises all, selected, filtered, and rerun Testing view requests. F02 and F03 violate that promise. Fix the implementation and add real Testing API request cases; do not use the passing flat fixture as evidence for arbitrary project layouts.

The Select Project or Workspace instructions imply the user can change the remembered project. F01 prevents the command-palette flow after a selection exists. Add an explicit project-switch regression.

The zero-configuration F5 paragraph says the owning project is discovered for a program main. F09 shows this depends on the spelling of valid TOML, rather than the manifest's meaning. Fix parsing/ownership resolution.

The terminal behavior paragraph promises keyboard and mouse bridging for integrated and external terminals. F05-F07 show missing chunking, unsupported-sequence recovery, mouse movement, and batch-size coverage. These are implementation defects, not reasons to redefine terminal input semantics.

## Documentation and test improvements

- State local-only operation and the behavior in a remote workspace directly. Change the error wording to identify an unsupported remote workspace rather than assuming where the extension host runs.
- Document the trust policy explicitly and test a Restricted Mode launch. The current suite passes `--disable-workspace-trust`, so it cannot verify this policy.
- Test the declared minimum VS Code version as well as the pinned host version, or deliberately raise the declared minimum after a compatibility decision. Keep the type package and runtime promise aligned.
- Remove stale development-only wording from the extension API comments in `src/extension.ts`; startup uses an installed selected toolchain.

No user-facing documentation or API declarations were changed during this review.
