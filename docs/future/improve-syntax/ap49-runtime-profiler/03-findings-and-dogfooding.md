# AP49.3: Findings, documentation and dogfooding

Package: [AP49](README.md). Status: in progress; findings and documentation delivered.
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
regression tests; record collection-policy observations for AP48 without changing
Q42 defaults.

## Documentation

Add `docs/pascal/tools/profiler.md` with the option, report fields, attribution
limits and an agent workflow; link it from the tools index and `fpas run --help`.

## Acceptance

Both dogfooding reports name the dominant cost in their findings. Documentation
matches the implemented fields. Workspace format, strict Clippy and tests pass.
