#ifndef FPAS_CUI_TABLE_H
#define FPAS_CUI_TABLE_H

CUI_API int fpas_cui_table_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const int *kinds, const float *widths, const int *alignments, const int *sortable, int count,
    int height_kind, float height, int striped, int compact, int selectable, int enabled,
    int *sort, int *descending, int focus_header, int *headers);
CUI_API int fpas_cui_table_row(cui_ctx *ui, const char *id, int selected, int enabled, int focus, int *node);
CUI_API void fpas_cui_table_row_end(cui_ctx *ui);
CUI_API void fpas_cui_table_cell(cui_ctx *ui, const char *text);
CUI_API void fpas_cui_table_end(cui_ctx *ui, int node);

#endif
