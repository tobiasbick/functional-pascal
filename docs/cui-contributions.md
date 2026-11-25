# CUI contribution review notes

Use these checks when preparing CUI fixes. Read the review against the current
PR commit and verify each finding in the code or with a small reproduction.

## Contribution workflow

Read these notes before changing CUI interaction or layout code or preparing
an upstream contribution. Establish whether a defect or missing capability
belongs to CUI or its FPAS integration and explain the impact.

Before drafting a contribution, read the current upstream
[issue form](https://github.com/yappologistic/CUI/tree/master/.github/ISSUE_TEMPLATE)
or [pull request template](https://github.com/yappologistic/CUI/blob/master/.github/pull_request_template.md).
Use its section order and complete the required fields, including when submitting
through the CLI or API. Follow the upstream
[contributing guide](https://github.com/yappologistic/CUI/blob/master/docs/CONTRIBUTING.md)
for authorship, AI disclosure and evidence.

For confirmed upstream findings, ask the user whether to open an issue and
whether to prepare and contribute a fix. Obtain explicit approval before
publishing an upstream issue or pull request.

## Tracking FPAS workarounds

Give each confirmed defect a stable local `CUI-NNN` ID and anchor in
[CUI upstream bugs](future/cui-upstream-bugs.md). Put the ID and the document
path with its anchor beside the FPAS workaround. Add upstream issue and PR
links to the same entry when available. Once the vendored CUI snapshot
contains the fix, verify the reproduction and remove the workaround and its
entry. Keep the IDs of the remaining entries.

## Confirming upstream fixes

Post confirmations on upstream CUI issues through the ChatGPT Codex
Connector. Verify that GitHub shows “with ChatGPT Codex Connector” next to
the author.

Include the tested CUI commit and the behavior you verified. If you removed a
local workaround, say so. Omit the downstream project's name. Leave issue
closure to the upstream maintainers, including for issues we opened.

## Interaction and layout

- Test nearby keys independently when changing a keyboard guard. Cover Enter
  and Space both as openers and while the popup is open. Exercise Escape
  dismissal separately.
- Queue a mouse press and an arrow key in the same frame when deferring a
  focus change. Apply the deferred move only while its originating control
  still has focus.
- When an open popup owns the keyboard, verify that its navigation keys leave
  the underlying widget's value alone. For range sliders, test that Space
  visibly distinguishes the active thumb even when both thumbs overlap.
- For focus rendering, inspect the pixels at the expected target and verify
  that the old ring disappears after focus moves.
- Base responsive layout on the space offered by the container. In a
  content-sized box, the control's previous width can keep it stuck in the
  compact layout. Exercise narrow, wide, and narrow again in the same
  instance.

## Pull request checks

- CodeRabbit requires 80% docstring coverage. Put a purpose comment using
  `/** ... */` immediately above each changed C definition in the diff,
  including the test runner if touched.
