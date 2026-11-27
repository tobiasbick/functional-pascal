// [Documentation](../../../../../docs/ui/gui/pickers/time-picker.md)
#ifndef FPAS_TIME_PICKER_H
#define FPAS_TIME_PICKER_H

/** Draws CUI's time picker.
 * @param ui Current drawing context.
 * @param id Stable field identity.
 * @param value Current time in minutes since midnight.
 * @param label Optional caption.
 * @param width_kind Content, fill, fixed or percentage in that order.
 * @param width Fixed logical width or percentage, when selected.
 * @param twelve_hour Whether to show the AM/PM column.
 * @param step Spacing between minute choices, from one to sixty.
 * @param enabled Whether the picker accepts input.
 * @param focus Whether to request trigger focus.
 * @param nodes Optional 87-node array for the field, 24 hours, 60 minutes and two periods.
 * @return Proposed time in minutes since midnight. */
CUI_API int fpas_cui_time_picker(cui_ctx *ui, const char *id, int value, const char *label,
    int width_kind, float width, int twelve_hour, int step, int enabled, int focus, int *nodes);
#endif
