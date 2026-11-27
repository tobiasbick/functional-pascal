# Navigation

Use the model to track the current destination, tab, page or workflow step.
Tabs switch content within a view. A sidebar switches between destinations.

| Widget | When to use it | Hosts |
| --- | --- | --- |
| [Sidebar navigation](sidebar.md) | Group application destinations under headings. | GUI and TUI |
| [Tabs](tabs.md) | Switch between sections within a view. | GUI |
| [Breadcrumb](breadcrumb.md) | Show the current location and its ancestors. | GUI |
| [Pagination](pagination.md) | Move through a collection one page at a time. | GUI |
| [Stepper](stepper.md) | Show progress through an ordered workflow. | GUI |

## Navigation gallery page

The page groups examples in three [native cards](../feedback/card.md).
The Tabs card uses a gap of 16 logical units. The Breadcrumb and pagination
card and the Stepper card each use a gap of 18 logical units.

The underline tabs update the label in the content column. That column uses
a gap of 6 and a vertical inset of 4 logical units. The three pill tabs show
icons and keep their selection independent of the underline tabs.

Selecting a breadcrumb ancestor shows its caption in a [toast](../overlays/toast.md).
The current location is informational. The pagination example starts at
page one and has twelve pages.

The stepper starts at Workspace. A [wrapping row](../layout/row.md) places
the Back and Next buttons 8 logical units apart. At Done, Next becomes
Finish and keeps its keyboard focus. Activating Finish shows the All set
success toast.

## See also

- [UI handbook](../README.md)
- [Reader](../reader/README.md)
- [App chrome](../app-chrome/README.md)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Navigation/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Navigation/Update.fpas)
- [View.fpas](../../../../examples/ui/gui/gallery/Pages/Navigation/View.fpas)
- [NavigationContract.fpas](../../../../tests/Fixtures/gui-host/NavigationContract.fpas)
