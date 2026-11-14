#ifndef FPAS_DATE_RANGE_CALENDAR_H
#define FPAS_DATE_RANGE_CALENDAR_H

/** Draws CUI's inline range calendar and reports a controlled selection.
 * @param ui Current drawing context.
 * @param id Stable calendar identity.
 * @param start Start day number, or -1. Receives the proposed start.
 * @param end End day number, or -1. Receives the proposed end.
 * @param months Number of visible months, one or two.
 * @param enabled Whether the calendar accepts input.
 * @param root Output native root node.
 * @param parts Output day numbers and navigation codes for visible controls.
 * @param nodes Output native nodes corresponding to parts. Both arrays hold 72 entries.
 * @param count Output number of tracked parts.
 * @return Whether CUI proposed a representable selection. */
CUI_API int fpas_cui_range_calendar(cui_ctx *ui, const char *id, int *start, int *end,
    int months, int enabled, int *root, int *parts, int *nodes, int *count);
#endif
