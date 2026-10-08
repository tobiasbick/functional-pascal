# AP02.4: JSON diagnostic output

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

`fpas check`, `build`, `run`, and `test` accept `--diagnostics json` and emit
UTF-8 JSON Lines on stderr. Records expose code, severity, phase, message,
source, location, expected, found, and hint. Unknown locations are null.

Progress and summaries do not enter the JSON diagnostic stream. Program stdout
and exit statuses retain their meaning. Program stderr is a distinct
`program-output` event. Diagnostic mode crosses runner boundaries without
consuming application arguments; native programs also accept
`FPAS_DIAGNOSTICS=json`. Test workers have a separate bounded stderr buffer;
overflow emits a complete truncation event without changing the test outcome.

## Regression coverage

Real-process tests cover all four commands, imported and location-free errors,
Unicode, child stderr, worker capture, truncation and startup failures.
See [CLI documentation](../../../pascal/program-structure/cli.md) and the
[diagnostic envelope](../../../pascal/tools/diagnostics.md).
