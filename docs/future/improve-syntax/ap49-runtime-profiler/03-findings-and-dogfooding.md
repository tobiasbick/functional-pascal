# AP49.3: Findings, documentation and dogfooding

Package: [AP49](README.md). Status: complete.
Prerequisite: [AP49.2](02-profile-report.md).

## Findings

Findings are deterministic rules over the report, each with the evidence numbers:

- collection time is at least 10 % of sampled execution time;
- classification visits on average at least 1,000 allocations per call;
- root publication and safe points take at least 10 % of sampled execution time;
- time-slice yields exceed spawned tasks by more than 100 times;
- pool workers are idle for at least half of their available time while tasks exist;
- one routine takes at least half of the sampled time;
- a top routine spends at least ten times the average time per instruction.

Each finding names the counter or routine and a concrete next step, such as the
relevant CLI option or source location. No finding changes runtime behavior.

## Dogfooding

Profile the headless Mandelbrot benchmark and a live graph of records with
closures. Fix confirmed runtime defects whose cause the report identifies, with
regression tests. Collection-policy changes need user approval because Q42 belongs
to AP48.

## Dogfooding results

- Live graph of records with closures: the report attributed 98 % of time to
  the building routine's `PushInPlace` line, at 177 times the average time per
  instruction, and classification at 1,667 visits per call. `ArrayPush`,
  `IndexSet`, `StoreField` and `UpdateRecord` revalidated the whole destination
  graph on every store. They now validate only the stored value; building
  20,000 handlers dropped from 39 s to 11 ms. A CLI test asserts linear
  classification work through the report counters.
- The same program then reported collection at 93 % of execution time: with
  20,000 live closures the periodic poll ran 144 passes tracing 5.6 million
  values (424 ms); with polling disabled, 4 passes (28 ms). Pass cost grows with
  the live graph while the poll interval was fixed. With user approval, the Q42
  thresholds became minimums that grow with the last traced graph (one
  allocation or mutation, two batches per traced value). Profiled afterwards:
  6 passes and 46 ms instead of 144 passes and 456 ms; with 100,000 live
  closures 60 ms instead of 3.6 s. Cycle churn beside 20,000 live closures went
  from 968 ms to 707 ms (153 to 20 passes) at a peak of 32 MB instead of 22 MB,
  because unreachable cycles may now grow to about the live graph's size.
  Mandelbrot and the VM benchmarks are unchanged. See
  [AP48.8](../ap48-local-cycle-collector/08-memory-architecture.md#phase-0-measure-the-corrected-baseline).
- Headless Mandelbrot: about 62 % of worker time is in the `Complex` record
  methods, which allocate per iteration, and pool workers are idle 77 % of the
  time because each frame waits for its slowest row. Both are properties of the
  example, not runtime defects.

## Documentation

Add `docs/pascal/tools/profiler.md` with the option, report fields, attribution
limits and an agent workflow; link it from the tools index and `fpas run --help`.

## Acceptance

Both dogfooding reports name the dominant cost in their findings. Documentation
matches the implemented fields. Workspace format, strict Clippy and tests pass.
