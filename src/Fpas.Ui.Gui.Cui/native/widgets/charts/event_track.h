#ifndef FPAS_CUI_EVENT_TRACK_H
#define FPAS_CUI_EVENT_TRACK_H

/** Editable-marker settings: minimum/maximum are finite axis bounds; step and
 * label_step control snapping and labels. format supplies numeric formatting.
 * width_kind/width define the FPAS extent; units selects numeric (0) or time (1).
 * flags: bit 0 disallows adding, bit 1 disallows removing, bit 2 hides the axis,
 * bit 3 enables input, bit 4 requests focus. label and label_user must remain
 * valid through frame presentation. */
typedef struct {
    double minimum, maximum, step, label_step;
    const char *format;
    float width;
    int width_kind, units, flags;
    cui_axis_label_fn label;
    void *label_user;
} fpas_event_options;
/** Draws an editable event track and writes its proposed sorted markers.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param times Mutable sorted marker buffer with capacity entries.
 * @param count Current marker count; receives the proposed count.
 * @param capacity Buffer capacity, at least the current count.
 * @param o Axis and editing settings.
 * @param result Five integers: selected index (input/output), added index,
 * removed index, dragging flag, and enabled changed flag. */
CUI_API void fpas_cui_event_track(cui_ctx *ui, const char *id, double *times, int *count,
    int capacity, const fpas_event_options *o, int *result);

#endif
