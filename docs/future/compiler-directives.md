# Future: Compiler Directives

> **Implemented.**  See `docs/pascal/12-compiler-directives.md` for the current specification.

## Implemented

- ~~**Conditional compilation**~~ — `{$IFDEF}`, `{$IFNDEF}`, `{$ELSE}`, `{$ENDIF}` — **implemented**
- ~~**Symbol management**~~ — `{$DEFINE name}`, `{$UNDEF name}` — **implemented**
- ~~**Include files**~~ — `{$I filename}` / `{$INCLUDE filename}` — **implemented** (project mode only; single-file mode emits an error)
- **Compiler switches** — `{$R+}`, `{$O+}`, etc. — accepted but have no effect (diagnostic emitted)
