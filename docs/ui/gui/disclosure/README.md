# Disclosure

The Disclosure gallery page includes controls that reveal more detail and
show progress within a page. These examples require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Banner](banner.md) | Announce a change in the page layout. |
| [Hover card](hover-card.md) | Preview a person's profile or other detail at a trigger. |
| [Link](link.md) | Activate a named destination inside text. |
| [Avatar](avatar.md) | Show initials or an image and optional presence. |
| [Confirm popover](confirm-popover.md) | Ask a short question next to the trigger. |
| [Expandable text](expandable-text.md) | Collapse a long passage with a Show more link. |
| [Expandable card](expandable-card.md) | Place collapsible details beneath a summary. |
| [Countdown](countdown.md) | Display the remaining time from the application clock. |
| [Onboarding checklist](onboarding-checklist.md) | Show completion and the next available step. |
| [Undo toast](undo-toast.md) | Give the user time to reverse an operation. |
| [Veil](veil.md) | Visually cover a picture or spoiler until revealed. |

## Disclosure gallery page

The announcement sits above the section headings. The profile hover card
opens from a link in a wrapping sentence. The confirmation opens from the
branch button and updates the sample result in the same row.

The Billing and Security cards share a column. Billing uses wrapped details
and actions. The two countdowns share a centered row. The checklist sits in
a 460-unit box and opens the first incomplete step.

The archive button shows a timed Undo toast. The last panel contains three
veils over a generated picture, a sample identifier and a spoiler.

## See also

- [UI handbook](../README.md)
- [Overlays](../overlays/README.md)
- [Disclosure in the gallery](../../../../examples/ui/gui/gallery/Pages/Disclosure/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#disclosure-evidence)

## Source files

- [Clock.fpas](../../../../examples/ui/gui/gallery/Pages/Disclosure/Clock.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Disclosure/State.fpas)
- [DisclosureContract.fpas](../../../../tests/Fixtures/gui-host/DisclosureContract.fpas)
- [DisclosureFeedbackContract.fpas](../../../../tests/Fixtures/gui-host/DisclosureFeedbackContract.fpas)
- [CuiDisclosureTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiDisclosureTests.cs)
