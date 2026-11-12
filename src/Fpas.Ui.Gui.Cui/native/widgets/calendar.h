#ifndef FPAS_CALENDAR_H
#define FPAS_CALENDAR_H

/** Draws the native single-date calendar.
 * @param ui Current drawing context.
 * @param id Stable calendar identity.
 * @param value Current day number, or minus one when empty.
 * @param enabled Whether the calendar accepts input.
 * @param focus Zero for no request, one for previous month or two for next month.
 * @param root Receives the calendar's root node.
 * @param parts Optional 44-entry array receiving day and navigation codes.
 * @param nodes Corresponding native nodes, also with 44 entries.
 * @param count Receives the number of reported parts.
 * @return The proposed day number, or the original value. */
CUI_API int fpas_cui_calendar(cui_ctx *ui, const char *id, int value, int enabled, int focus,
    int *root, int *parts, int *nodes, int *count);
#endif
