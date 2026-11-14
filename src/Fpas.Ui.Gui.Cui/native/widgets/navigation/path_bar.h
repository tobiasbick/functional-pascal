#ifndef FPAS_CUI_PATH_BAR_H
#define FPAS_CUI_PATH_BAR_H
CUI_API int fpas_cui_path_bar(cui_ctx *ui, const char *id, const char *const *segments, int count,
    const char *text, const char *icon, const char *placeholder, int size, int width_kind, float width,
    int flags, int focus, const char **submitted, int *parts, int *root, uint32_t *retained_key);
CUI_API int fpas_cui_path_menu_begin(cui_ctx *ui, const char *id);
CUI_API void fpas_cui_path_menu_end(cui_ctx *ui);
CUI_API int fpas_cui_path_menu_item(cui_ctx *ui, const char *label, int index);
#endif
