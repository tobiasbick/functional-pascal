#ifndef FPAS_CUI_RATIO_SLIDER_H
#define FPAS_CUI_RATIO_SLIDER_H
CUI_API int fpas_cui_ratio_slider(cui_ctx *ui, const char *id,
    const char *first, const char *second, float *value, int tone, float step, float height, int width_kind, float width, int flags, int focus, int *parts);
#endif
