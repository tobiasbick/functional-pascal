# UMB-90 scope and decisions

## Shared invariants

1. Runtime identities never depend on display text or adapter-local handles.
2. A rejection changes no worker, scheduler result, waiter, stop generation,
   or adapter state.
3. JSONL and DAP call the same session or host operation. VS Code maps to
   that adapter behavior.
4. No FPAS syntax, semantics, or language documentation changes are in scope.
5. Reload observes all-stop quiescence from `UMB-40A` and debuggee-channel
   separation from `UMB-50`.
6. Memory, disk, snapshot count, and retention stay bounded.
7. The recording capture log from `UMB-80` is not a live-image snapshot store.

## Current ownership inventory

These are inventory facts for `U90-00`, not acceptance of later children.

- Debug sessions are launch-owned and all-stop. The session executable is an
  immutable `Arc<VerifiedExecutable>` shared by workers. `FunctionId` values
  are local to that image.
- JSONL advertises `record_replay: false`. DAP advertises
  `supportsStepBack: false`. Describe reports `replayable: false`.
- Capture exists after `record` / `fpas/record`, keeps at most 4,096 events,
  writes no files, and retains no recording snapshots.
- JSONL advertises `hot_reload: false`. Named `reload` / `image.replace` and
  DAP `fpas/reload` rejects exist. No compatibility classifier or recoverable
  old image exists in `fpas-vm`.
- Attach, remote, and native debugging were rejected by `UMB-60`. Replay
  remains unsupported after `UMB-80`.

## Frozen hot-reload-off contract (`U90-01`)

Hot reload stays off until later children name accepted updates. A named
reject changes no stack, frame, task, or live executable. Missing rules
below are tests or recorded bounds, not a live-image claim.

Observable today on the current launch-owned path:

- One immutable `Arc<VerifiedExecutable>` shared by workers. `FunctionId`
  values stay image-local.
- JSONL `hot_reload: false`. DAP does not advertise hot reload. Named JSONL
  `reload` / `image.replace` and DAP `fpas/reload` rejects do not resume or
  replace the image.
- Recording capture from `UMB-80` stays a stop/input log. It is not a
  snapshot store for live-image replacement.

Not reloadable until classified, rejected, or migrated by later children:

- Active and inactive function bodies
- Record, enum, and global layouts
- Closures, capture cells, and task identities
- Debug metadata, source maps, and sequence points
- Newly entered anonymous closures (`UMB-10B`)

Named rejects without resume or mutation:

- JSONL `reload` and `image.replace`
- DAP `fpas/reload`

## `UMB-90A` — compatibility classification

- Begins only after `U90-01` freezes hot-reload-off and named rejects.
- The proven accepted subset is `unchanged` and `inactive_function_body`.
- Named rejects are `active_function_body`, `record_layout`, `enum_layout`,
  `global_layout`, `closure_capture`, `task_identity`, `function_set`,
  `anonymous_closure`, `entry_point`, and `debug_metadata`.
- Classification compares a candidate image with the live executable and
  current stacks. It does not replace the live `Arc<VerifiedExecutable>`.
- Live function values and capture cells are not heap-scanned; capture-count
  and capture-source mismatches are `closure_capture`. New capturing functions
  are `anonymous_closure`; `UMB-10B` stays blocked.
- JSONL `reload.classify` and DAP `fpas/reloadClassify` name those classes
  without a second compiled candidate and report `applied: false`.

## `UMB-90B` — reject before commit

- Begins only after compatibility can name accepted and rejected updates.
- Incompatible updates are rejected before the live program image changes.
- A named reject changes no stack, frame, task, or adapter state.

## `UMB-90C` — versioned live image and rollback

- Begins only after reject-before-commit is proven.
- A recoverable old image stays available until the new image commits.
- Default paths must not retain unbounded images or snapshots.
- JSONL, DAP, and VS Code report the same accepted and rejected changes.

## Out of scope

- Attach/remote (rejected by `UMB-60`).
- Recording replay (unsupported after `UMB-80`).
- In-call host interruption (rejected by `UMB-50D`).
- Non-stop execution (rejected by `UMB-40D`).
- Unauthenticated exposure of sources, recordings, or host metadata.
