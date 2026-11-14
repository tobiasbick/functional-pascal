#ifndef FPAS_WHEEL_PICKER_H
#define FPAS_WHEEL_PICKER_H

/** Draws CUI's wheel picker.
 * @param ui Current drawing context.
 * @param id Stable wheel identity.
 * @param items Choice captions copied to the native frame.
 * @param count Number of choices, at least one.
 * @param index Current zero-based selection.
 * @param width Logical width, or zero for CUI's default.
 * @param rows Number of visible rows.
 * @param loop Whether the selection wraps at the ends.
 * @param join_left Whether the band joins the preceding wheel.
 * @param join_right Whether the band joins the following wheel.
 * @param enabled Whether the wheel accepts input.
 * @param focus Whether to request wheel focus.
 * @param node Receives the native wheel node.
 * @return The proposed zero-based selection. */
CUI_API int fpas_cui_wheel_picker(cui_ctx *ui, const char *id, const char *const *items, int count,
    int index, float width, int rows, int loop, int join_left, int join_right, int enabled,
    int focus, int *node);
#endif
