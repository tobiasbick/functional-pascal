# AP48.7: Automatic scheduling and acceptance

Package: [AP48](README.md). Status: not started. Effort: medium to large.
Prerequisite: [AP48.6](06-cycle-collector.md) complete.

## Work

Request collection using per-domain cell allocation and mutation pressure, serviced
at the AP48.5 safe points. Include periodic checks while execution continues, before
parking, at termination and after external-root release. A graph can become garbage
when a register or handle drops without a new cell allocation; scheduling must cover
that case. Avoid a collector thread or global pause unless a later design explicitly
requires one. No public FPAS collection controls are introduced.

Choose initial thresholds from controlled measurements rather than arbitrary defaults.
Before activation, record concrete workload-specific throughput, memory and pause
budgets in this work package. Compare the same workloads with the original baseline,
ownership enforcement alone and automatic collection. Separate graph checking costs
from collection costs. These measurements are future work, not current results.

## Acceptance workloads

| Workload | Pass criterion |
| --- | --- |
| Long-running loop creates and drops closure cycles | Repeated runs return live local cells to a stable envelope after collection |
| Live mutable closures survive repeated passes | Results, alias identity and state match execution without collection |
| Deep/wide graphs and many immutable transfers | No stack overflow; measured costs meet recorded budgets |
| Migration, channel waits, cancellation and retries | No deadlock, stranded task or retained abandoned domain |
| External roots released after task completion | Only rooted graphs survive; final release triggers cleanup without the original worker |
| Busy collector and independent worker | Independent worker progresses during collection |
| Callback/session/debugger churn | No accumulating detached cycles, stale roots or owner collisions |

Measure live allocations separately from pooled capacity and process resident memory.
Record pause distributions and the largest tested graph. Do not claim hard real-time
limits or infer reclamation solely from RSS. If budgets fail, retain explicit
collection and revise the design before automatic activation.

## Owners, documentation and verification

Integrate scheduling in domain counters, dispatch and lifecycle code. Reuse existing
runtime benchmark infrastructure and closure/concurrency regression owners. Add only
workloads that distinguish a credible failure or cost; no implementation-mirroring
tests or synthetic public language APIs.

Complete all applicable checks from the [development process](../development-process.md#required-checks),
including Rust formatting/build/workspace tests, the FPAS suite and affected projects.
Check portable/cached artifacts, embedding consumers and debugging behavior.
Review the result against the contract and repository standards.

Update the closure, task and variable-reference documentation, Rust embedding
contracts and compact reference as applicable. Explain reference counting plus local
cycle collection and the absence of a global execution lock. Completion requires
recorded budget results and passing package acceptance, not merely a successful build.
