#ifndef FPAS_CUI_TAB_BAR_H
#define FPAS_CUI_TAB_BAR_H
CUI_API int fpas_cui_tab_bar(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *badges, int count, int *selected, int width_kind, float width, int flags, int focus, int *parts);
#endif
