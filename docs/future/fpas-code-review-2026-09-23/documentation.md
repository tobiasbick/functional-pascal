# Documentation findings

## D01, P2: Copyable project examples omit public exports

Source: `docs/pascal/program-structure/projects.md:204` and `:244`, with callers at lines 195 and 273.

Both `MyApp.Math.Add` and `Acme.Math.Add` are declared with `function Add`, while programs in other units call them. Declarations are private by default. The same document explains that rule at line 125, but its examples do not follow it.

The first example's manifest, main program, and math unit were extracted unchanged into a scratch project. Running `fpas check` returned:

```text
4:11: error[F2003]: Unknown function or procedure `Add`
```

Add `public` to the exported functions when implementing the documentation fix. This corrects examples to the current language; it does not propose changing visibility semantics. Add a test that compiles extracted examples or maintain a small set of checked documentation projects. Existing runnable library examples do not catch drift in these separately written Markdown snippets.

## D02, P3: The example index describes an obsolete shutdown strategy

Source: `examples/README.md:279`; implementation at `examples/network/tcp_parallel_echo_server.fpas:57`, `:111`, and `:134`.

The index says the server retains and joins unfinished workers after a one-second close attempt and provides no hard process-exit guarantee. The implementation creates a server lifetime with process escalation enabled, owns the listener, observes shutdown signals, and uses the remaining shutdown deadline when closing the task group. The failure path leaves the watchdog armed; successful shutdown finishes the lifetime.

`examples/network/README.md` and the example table already describe process escalation. Update the stale paragraph to match those descriptions and distinguish orderly completion from watchdog escalation. Keep timeout claims tied to the runtime contract and existing shutdown tests.

## Contract mismatches recorded with their causes

- `apps/notes/README.md:22` promises that directory scanning stays inside the selected directory. F01 reproduces a violation. Fix the directory lookup rather than weakening the promise.
- `docs/pascal/std/network/uri.md` describes an HTTP/HTTPS parser. F06 shows that an explicit port bypasses the scheme check. Fix the parser.
- SSE event-size validation returns errors, but the retained-state behavior in F07 needs a defined error-state contract and an implementation fix. Do not claim a measured memory ceiling from the existing error tests.

No current language or API documentation was edited during this review. All proposed work remains in this review directory.
