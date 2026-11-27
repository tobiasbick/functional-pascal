# Reader

Reader connects an outline to a scroll view with named destinations. CUI
handles jumps and tracks the active section as the article scrolls. Both
widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Table of contents](table-of-contents.md) | Jump to a section and track the current heading. |
| [Scroll anchor](scroll-anchor.md) | Name a destination in the scrolling content. |

## Reader gallery page

The article and outline share a row with a gap of 28 logical units. The
article fills the remaining width and uses a 560-unit viewport with edge
fades. Its anchors, headings and paragraphs are direct children of the
viewport with a 14-unit gap. A gap of 200 units follows the last section.

The outline column is 200 logical units wide. It contains eight entries in
article order, including two indented entries under Install.

## See also

- [UI handbook](../README.md)
- [Navigation](../navigation/README.md)
- [Scroll view](../layout/scroll-view.md)
- [Reader in the gallery](../../../../examples/ui/gui/gallery/Pages/Reader/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#reader-evidence)

## Source files

- [ReaderContract.fpas](../../../../tests/Fixtures/gui-host/ReaderContract.fpas)
