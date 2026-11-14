#ifndef FPAS_CUI_TOOLBAR_H
#define FPAS_CUI_TOOLBAR_H
CUI_API void fpas_cui_toolbar_begin(cui_ctx *ui, const char *id, int style, int no_border, int enabled);
CUI_API void fpas_cui_toolbar_end(cui_ctx *ui);
CUI_API int fpas_cui_toolbar_button(cui_ctx *ui, const char *icon, const char *label, const char *tooltip,
    const char *shortcut, int toggle, int tone, int flags);
CUI_API int fpas_cui_toolbar_radio(cui_ctx *ui, const char *id, const char *const *icons,
    const char *const *labels, int count, int value);
CUI_API int fpas_cui_toolbar_cycle(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int value, const char *tooltip);
#endif
