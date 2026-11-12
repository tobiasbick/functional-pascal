#ifndef FPAS_DATE_RANGE_PICKER_H
#define FPAS_DATE_RANGE_PICKER_H

/** Draws CUI's native range picker.
 * @param ui Current drawing context.
 * @param id Stable picker identity.
 * @param start Start day number, or -1. Receives the proposed start.
 * @param end End day number, or -1. Receives the proposed end.
 * @param label Optional field caption.
 * @param placeholder Optional empty-value caption.
 * @param width Logical field width, or zero for the native default.
 * @param presets Whether to show CUI's relative date presets.
 * @param enabled Whether the picker accepts input.
 * @param focus Whether to request trigger focus.
 * @param trigger Output native trigger node.
 * @param parts Output day numbers, preset codes and navigation codes.
 * @param nodes Output native nodes corresponding to parts. Both arrays hold 72 entries.
 * @param count Output number of tracked parts.
 * @param open Whether the popup accepts input.
 * @return Whether CUI proposed a representable selection. */
CUI_API int fpas_cui_range_picker(cui_ctx *ui, const char *id, int *start, int *end,
    const char *label, const char *placeholder, float width, int presets, int enabled, int focus,
    int *trigger, int *parts, int *nodes, int *count, int *open);
#endif
