#ifndef FPAS_CUI_MENU_BAR_H
#define FPAS_CUI_MENU_BAR_H

CUI_API void fpas_cui_menu_bar_begin(cui_ctx *, const char *, int, int);
CUI_API void fpas_cui_menu_bar_end(cui_ctx *);
CUI_API int fpas_cui_menu_bar_menu(cui_ctx *, const char *, int, float, int, const char *const *,
    const char *const *, const char *const *, const char *const *, const int *, int, int *, int *, int *);

#endif
