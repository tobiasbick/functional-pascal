#ifndef FPAS_CUI_EVENT_TRACK_H
#define FPAS_CUI_EVENT_TRACK_H

typedef struct {
    double minimum, maximum, step, label_step;
    const char *format;
    float width;
    int width_kind, units, flags;
} fpas_event_options;
CUI_API void fpas_cui_event_track(cui_ctx *ui, const char *id, double *times, int *count,
    int capacity, const fpas_event_options *o, int *result);

#endif
