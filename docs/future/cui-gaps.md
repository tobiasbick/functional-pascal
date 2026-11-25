# CUI gaps

The [CUI widget inventory](../cui-widget-inventory.md) records accepted widgets
and current terminal coverage. Desktop host and publishing work is tracked in
[Desktop GUI application gaps](gui-app-gaps.md). Confirmed defects with FPAS
workarounds are tracked in [CUI upstream bugs](cui-upstream-bugs.md).

## Drag and drop within a window

Add a shared CUI interface that lets widgets opt in as drag sources, drop
targets, or both. A source specifies a payload type and the allowed copy or
move actions. CUI owns a copy of the payload bytes for the drag. An application
can pass stable IDs for objects it owns. A target can inspect the payload
while hovered and decide whether, where, and how to accept it.

CUI shows a lifted row preview, a thin insertion line for row drops, and
feedback for accepted and rejected targets using the active theme. Scroll
when the pointer nears the edge of a scrollable target. Report an accepted
drop once on release so the application can update its data. Escape, focus
loss, and release outside a valid target cancel the drag.

- [ ] Reorder rows within one DataGrid, including beyond its visible range.
- [ ] Move a row between two DataGrids using the shared source and target interface.
- [ ] Drag an item from a TreeTable to an accepting Kanban column.

## Strict gallery pixel parity

- [ ] Remove the remaining single-channel antialiasing differences in wide page
  composition. At `86d9781`, Linux software captures at 1240x1700, scale 1,
  differ by one 8-bit channel step at 182/184 pixels in the Inputs picker card
  (dark/light), 487 in dark Feedback progress and one in dark Loading. The
  measured wide picker positions differ by about 0.00012 logical points from
  direct CUI. Ten of twelve reviewed Inputs regions and four of six reviewed
  Feedback regions match exactly. Comparisons exclude the reserved sample email,
  unsynchronized activity animations and the unbound skeleton group. The native
  widget and calendar tests compare at identical positions and times and pass
  without a pixel tolerance. Keep strict whole-page parity open until the
  composition differences are resolved.

- [ ] Match the supplementary App chrome composition to CUI's documentation
  captures. At 1100 and 550 logical points, scale one, Software and OpenGL show
  remaining differences in the editor code and status alignment, the Docking
  viewport illustration and frame, and the boundary pixels of Split view.
  The direct native widget comparison is exact. Keep the assembled examples
  open until their corresponding regions also match.
