#ifndef FPAS_CUI_CURVE_EDITOR_H
#define FPAS_CUI_CURVE_EDITOR_H
typedef struct {
    const char *x_label, *y_label;
    float x_min, x_max, y_min, y_max, height, width, playhead;
    int width_kind, flags;
} fpas_curve_options;
CUI_API int fpas_cui_curve_editor(cui_ctx *ui, const char *id, cui_curve_key *keys, int *count, int capacity, int *selected, int *added, int *removed, const fpas_curve_options *settings);
#endif
