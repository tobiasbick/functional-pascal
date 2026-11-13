#ifndef FPAS_CUI_GRADIENT_SLIDER_H
#define FPAS_CUI_GRADIENT_SLIDER_H
CUI_API int fpas_cui_gradient_slider(cui_ctx *ui, const char *id,
    const char *label, float *value, float minimum, float maximum, int track, const float *positions, const uint32_t *colors, int count, int palette, uint32_t rgba, float step, const char *format, const char *display, const char *tool, const char *tooltip, int flags, int focus, uint32_t *sample, int *parts);
#endif
