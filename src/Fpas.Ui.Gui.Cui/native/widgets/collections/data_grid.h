#ifndef FPAS_CUI_DATA_GRID_H
#define FPAS_CUI_DATA_GRID_H
CUI_API void *fpas_cui_data_grid_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const float *widths, const int *flags, int columns, int rows, uint8_t *selected,
    int *state, int height_kind, float height, float row_height, int options,
    int *range, int *headers);
CUI_API int fpas_cui_data_grid_row(cui_ctx *ui, int row, const char *id);
CUI_API void fpas_cui_data_grid_cell(cui_ctx *ui, const char *text);
CUI_API int fpas_cui_data_grid_edit(cui_ctx *ui, char *value, int capacity);
CUI_API void fpas_cui_data_grid_end(cui_ctx *ui, void *handle, uint8_t *selected, int *state);
#endif
