# CUI contribution review notes

Use these checks when preparing CUI fixes. Read the review against the current
PR commit and verify each finding in the code or with a small reproduction.

## Interaction and layout

- Test nearby keys independently when changing a keyboard guard. Cover Enter
  and Space both as openers and while the popup is open. Exercise Escape
  dismissal separately. [Split-button review](https://github.com/yappologistic/CUI/pull/26#discussion_r4531482348)
- Queue a mouse press and an arrow key in the same frame when deferring a
  focus change. Apply the deferred move only while its originating control
  still has focus. [Toggle-group review](https://github.com/yappologistic/CUI/pull/46#discussion_r4543112083)
- When an open popup owns the keyboard, verify that its navigation keys leave
  the underlying widget's value alone. For range sliders, test that Space
  visibly distinguishes the active thumb even when both thumbs overlap.
  [Popup keyboard ownership](https://github.com/yappologistic/CUI/pull/47#discussion_r4543120107) ·
  [Overlapping thumbs](https://github.com/yappologistic/CUI/pull/47#discussion_r4543120099)
- For focus rendering, inspect the pixels at the expected target and verify
  that the old ring disappears after focus moves.
  [Breadcrumb and pagination review](https://github.com/yappologistic/CUI/pull/48#pullrequestreview-5865193759)
- Base responsive layout on the space offered by the container. In a
  content-sized box, the control's previous width can keep it stuck in the
  compact layout. Exercise narrow, wide, and narrow again in the same
  instance. [Pagination review](https://github.com/yappologistic/CUI/pull/49#discussion_r4543118260)

## Applied upstream fixes

- Copy-button focus, PR #24. CodeRabbit reported 0% docstring coverage against
  an 80% threshold. Purpose comments using `/** ... */` immediately above the
  affected C definitions brought the same check to 100%. Review the changed
  functions in the diff, including the test runner if touched. CUI applied the
  fix and documentation follow-up as `91fa783` and `c29878b`.
  [CodeRabbit report](https://github.com/yappologistic/CUI/pull/24#issuecomment-6779222285) ·
  [Upstream confirmation](https://github.com/yappologistic/CUI/pull/24#issuecomment-6798558699)
- Split-button selection, PR #26. CodeRabbit flagged the same docstring
  threshold and noticed that the Enter guard also made Space a no-op on an
  open menu. The maintainer applied the selection fix as `8d07cf5`. Preserve
  and test adjacent key behavior when routing one key to a menu.
  [CodeRabbit report](https://github.com/yappologistic/CUI/pull/26#issuecomment-6781148603) ·
  [Upstream confirmation](https://github.com/yappologistic/CUI/pull/26#issuecomment-6798559346)
