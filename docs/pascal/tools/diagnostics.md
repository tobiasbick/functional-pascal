# Diagnostics

Compiler diagnostics identify a problem by code, severity, and source
path/line/column/length. Correct the reported problem and compile again; a failed
compilation can stop before later problems are reported.

## Output and exit status

Normal CLI output writes diagnostics to stderr as
`path(line,column): error FPnnnn: message` or the corresponding warning form.
Successful compilation writes `Wrote path.dll` to stdout.

Use `--diagnostics json` for automation. It writes one stdout object containing
`success`, `output`, and `diagnostics`; each diagnostic contains `code`, `message`,
`severity`, and `location`. The output path is null when compilation fails.
Locations are one-based; command-line and input failures use a synthetic source
location when there is no source token to identify.

| Exit code | Meaning |
| --- | --- |
| 0 | Successful compilation or help |
| 1 | Compilation, input, or output failure |
| 2 | Invalid command-line arguments |

## Codes

| Codes | Meaning |
| --- | --- |
| FP0001 / FP0002 | Invalid CLI arguments / input-output failure |
| FP1001–FP1004 | Invalid character, string, separator, or unsupported numeric form |
| FP2001–FP2005 | Syntax/terminator error, missing type annotation, unsupported form, source/output-kind mismatch, or duplicate container |
| FP3001–FP3008 | Name, type, overload, binding, unsupported type, unused result, missing return, or unreachable statement |
| FP3009 / FP3010 | Non-writable assignment target / loop exit outside a loop |
| FP3011 / FP3012 | Private routine/member/constructor access / importing a program |
| FP3013 / FP3015–FP3017 | Nullable safety / property accessors, disposal, or exception-flow errors |
| FP3018 / FP3019 | Generic constraint failure / unsupported or unresolved callable signature |
| FP3021 / FP3022 | Invalid record declaration or visibility / invalid record construction or update |
| FP3023–FP3025 | Invalid union declaration, pattern, or coverage/unreachable case arm |
| FP3026 | Invalid/unsupported source generic declaration or explicit type-argument arity |
| FP3027 / FP3028 | Invalid contract / parameter modes, defaults, storage, or out flow |
| FP3029 / FP3030 | Invalid record interface implementation / async or await reference-lifetime error |
| FP3031 | Invalid alias, enum, distinct declaration, subrange bounds, or domain construction |
| FP3032 | Invalid global name or public global exposing private types |
| FP3033 | Nested routine environment created before captured bindings initialize |
| FP3034 | Invalid or ambiguous event selection/access, or event used as a delegate value |
| FP3035 | Invalid attribute class, target, duplicate, constant value, named member, or structural metadata |
| FP6001 | Documentation warning; severity follows the configured warning policy |
| FP4001 / FP4002 | Invalid/missing reference metadata / ambiguous reference identity |
| FP5001 | Invalid or unsupported SDK project configuration |
| FP5002 | Build-adapter input/output or cleanup failure |
| FP5003 | Compiler process failure or missing requested assembly |
| FP5004 | Compilation canceled or timed out through the build adapter |

## Warning policy

XML documentation diagnostics use `FP6001`. `--warn-as-error` promotes warnings;
`--warnings-as-errors` promotes selected codes, `--warnings-not-as-errors` excludes
selected codes from the global promotion, and `--no-warn` suppresses selected
codes. Code lists accept commas or semicolons. Promoted warnings fail compilation
and prevent publication of new outputs.

The project SDK reports diagnostics through MSBuild with their source locations.
See the [SDK configuration](../../build/msbuild.md) for project warning properties
and the [compiler CLI](../program-structure/cli.md) for direct invocation.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
