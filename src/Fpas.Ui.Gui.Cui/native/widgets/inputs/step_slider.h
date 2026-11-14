#ifndef FPAS_CUI_STEP_SLIDER_H
#define FPAS_CUI_STEP_SLIDER_H
CUI_API int fpas_cui_step_slider(cui_ctx *ui, const char *id,
    const char *label, int *value, int minimum, int maximum, const char *const *labels, int count, const char *format, const char *display, const char *tool, const char *tooltip, int flags, int focus, int *parts);
#endif
