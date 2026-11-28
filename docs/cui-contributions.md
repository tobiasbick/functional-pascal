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
publishing an upstream issue or pull request. Work through the
[CUI pull request checklist](future/cui-checklist.md) before you open or update
a pull request.

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
