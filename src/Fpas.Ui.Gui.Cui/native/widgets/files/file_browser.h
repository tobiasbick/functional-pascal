#ifndef FPAS_CUI_FILE_BROWSER_H
#define FPAS_CUI_FILE_BROWSER_H
CUI_API int fpas_cui_file_browser(cui_ctx *ui, const char *id, char *folder, int capacity,
    int width_kind, float width, int height_kind, float height, const char *filter,
    const char *const *places, int place_count, cui_file_list_fn source, void *user, int maximum,
    int revision, int flags, int focus, const char **selected, const char **picked, int *parts);
#endif
