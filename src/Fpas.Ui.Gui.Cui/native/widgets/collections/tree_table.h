#ifndef FPAS_CUI_TREE_TABLE_H
#define FPAS_CUI_TREE_TABLE_H
CUI_API int fpas_cui_tree_cell(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int depth, int *open, int enabled, int *part);
CUI_API void *fpas_cui_tree_table_begin(cui_ctx *ui, const char *id,
    const char *const *titles, const float *widths, const int *column_flags, int columns,
    const char *const *labels, const int *parents, const uint32_t *flags, int count,
    int *state, int height_kind, float height, float row_height, int options, int *node);
CUI_API int fpas_cui_tree_table_row(cui_ctx *ui, int *part);
CUI_API int fpas_cui_tree_table_name(cui_ctx *ui, const char *label, const char *icon);
CUI_API void fpas_cui_tree_table_end(cui_ctx *ui, void *handle, uint32_t *flags,
    int *state, int *events, int *load);
#endif
