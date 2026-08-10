# Local editor bug report

The extension collects no telemetry. Copy this template into a local note or a
repository issue when a problem can be reproduced. Remove machine-identifying
or sensitive data before sharing it.

```text
Title: <short observable problem>

Extension version: <for example 0.2.0>
Editor and version: <VS Code, Cursor, or compatible editor>
Host target: <for example win32-x64 or linux-x64>
FPAS context: <loose file, .fpasprj, or .fpasworkspace>

Steps to reproduce:
1. <smallest first step>
2. <next step>
3. <command or key such as F2, F12, Shift+F12, or Format Document>

Expected:
<observable expected result>

Actual:
<observable result and Fxxxx diagnostic codes>

Minimal FPAS source:
<small self-contained source or project layout>

Functional Pascal output:
<only the relevant sanitized lines>
```

Do not include usernames, hostnames, absolute home-directory paths, secrets,
or unrelated output. Replace absolute project roots with `<workspace>` and
extension installation roots with `<extension>`.

Before recording a new problem, run **Developer: Reload Window**, reproduce it
once with the current locally built VSIX, and check whether **Functional
Pascal: Restart Language Server** changes the result. A report stays local
unless a user explicitly decides to submit it.
