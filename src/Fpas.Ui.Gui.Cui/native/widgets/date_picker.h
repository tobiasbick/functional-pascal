#ifndef FPAS_DATE_PICKER_H
#define FPAS_DATE_PICKER_H

/** Draws the native field and its single-date calendar popup.
 * @param ui Current drawing context.
 * @param id Stable picker identity.
 * @param value Current day number, or minus one when empty.
 * @param label Optional field caption.
 * @param placeholder Optional caption for the empty field.
 * @param width Logical field width, or zero for CUI's default.
 * @param enabled Whether the field accepts input.
 * @param focus Whether to request trigger focus.
 * @param trigger Receives the native field node.
 * @param parts Optional 45-entry array receiving the same part codes as fpas_cui_calendar.
 * @param nodes Corresponding native nodes, also with 45 entries.
 * @param count Receives the number of reported popup parts.
 * @return The proposed day number, or the original value. */
CUI_API int fpas_cui_date_picker(cui_ctx *ui, const char *id, int value, const char *label,
    const char *placeholder, float width, int enabled, int focus,
    int *trigger, int *parts, int *nodes, int *count);
#endif
