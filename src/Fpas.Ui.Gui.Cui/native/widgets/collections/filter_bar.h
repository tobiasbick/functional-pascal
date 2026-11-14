#ifndef FPAS_CUI_FILTER_BAR_H
#define FPAS_CUI_FILTER_BAR_H
CUI_API int fpas_cui_filter_bar(cui_ctx *ui, const char *id, const char *const *names,
    const char *const *icons, const char *const *values, const int *counts, int count,
    uint32_t *selected, char *search, int capacity, int enabled, int focus, int *parts);
#endif
