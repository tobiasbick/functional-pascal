#ifndef FPAS_CUI_ASPECT_PICKER_H
#define FPAS_CUI_ASPECT_PICKER_H
CUI_API int fpas_cui_aspect_picker(cui_ctx *ui, const char *id,
    const char *label, int *value, const int *widths, const int *heights, const char *const *labels, int count, const char *custom, int base, const char *display, int flags, int focus, int *parts);
#endif
