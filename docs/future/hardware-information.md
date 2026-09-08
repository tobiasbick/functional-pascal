# Hardware information — idea

Status: idea only; no implementation plan or API decision.

FPAS programs could benefit from access to hardware information such as available CPU
parallelism and total or available RAM, for example when choosing the amount of concurrent work
or a memory budget.

The Mandelbrot investigation prompted this idea. Its current fix uses the existing VM worker
pool and does not require a public hardware-information API.

No unit names, signatures, platform coverage, milestones, or implementation work are proposed here.
