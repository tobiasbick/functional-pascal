# Feedback

Use the model to supply progress, status and data. CUI draws the indicators
and their transitions in the GUI. The TUI also supports determinate progress bars.

| Widget | When to use it |
| --- | --- |
| [Alert](alert.md) | Show an inline result or warning. |
| [Progress bar](progress-bar.md) | Show progress along a track or indicate work with no known total. |
| [Meter](meter.md) | Show consumption relative to capacity. |
| [Spinner](spinner.md) | Indicate ongoing work. |
| [Status spinner](status-spinner.md) | Show work in progress and its result. |
| [Skeleton](skeleton.md) | Reserve the shape of loading content. |
| [Skeleton group](skeleton-group.md) | Share one animation across placeholders. |
| [Timeline item](timeline-item.md) | Show a dated entry in an activity feed. |
| [Progress ring](progress-ring.md) | Show a percentage with a short caption inside a ring. |
| [Badge](badge.md) | Display a compact status label. |
| [Avatar](../disclosure/avatar.md) | Identify a person by initials or an image. |
| [Avatar group](avatar-group.md) | Fit several people and an overflow count into one row. |
| [Empty state](empty-state.md) | Explain an empty collection and offer an action. |
| [Grid](grid.md) | Arrange the feedback cards in responsive columns. |
| [Card](card.md) | Group related controls on a native surface. |

## Feedback gallery page

The Alerts section uses a [column](../layout/column.md) with a gap of 10
logical units. The storage alert includes a dismiss button. The gallery keeps
the alert hidden when the user leaves Feedback and returns.

Two responsive grids group the following cards. Each grid fits up to two
columns with a minimum width of 260 logical units and a gap of 16.

| Grid | Cards |
| --- | --- |
| First | Progress, Loading and Dither spinners |
| Second | Activity and Goals |

The Progress card uses a gap of 18 logical units. The other grid cards
use 14. The Goals row distributes the available space around its two rings.
The Activity card shows three timeline items. The final item omits the connector.

The Badges and avatars card has two wrapping [rows](../layout/row.md).
The team avatar group shows four people and a +3 overflow count. The final
card offers a New project action, which shows a success toast.

## See also

- [UI handbook](../README.md)
- [Toast](../overlays/toast.md)

## Source files

- `examples/ui/gui/gallery/Pages/Feedback/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Feedback/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Feedback/Update.fpas)
  - [View.fpas](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- `examples/ui/gui/gallery/Pages/Feedback/Widgets/`
  - [BadgesAndAvatars.fpas](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/BadgesAndAvatars.fpas)
  - [Loading.fpas](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/Loading.fpas)
- `tests/Fixtures/gui-host/`
  - [DisclosureFeedbackContract.fpas](../../../../tests/Fixtures/gui-host/DisclosureFeedbackContract.fpas)
  - [FeedbackContract.fpas](../../../../tests/Fixtures/gui-host/FeedbackContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Feedback/`
  - [CuiFeedbackTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Feedback/CuiFeedbackTests.cs)
