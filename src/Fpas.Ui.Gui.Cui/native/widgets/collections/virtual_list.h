#ifndef FPAS_CUI_VIRTUAL_LIST_H
#define FPAS_CUI_VIRTUAL_LIST_H

CUI_API int fpas_cui_virtual_list_begin(cui_ctx *ui, const char *id, int count, float row_height, int kind, float height, int enabled, int *first, int *last);
CUI_API void fpas_cui_virtual_list_end(cui_ctx *ui, int node);
CUI_API void fpas_cui_virtual_row_begin(cui_ctx *ui, int index);
CUI_API void fpas_cui_virtual_row_end(cui_ctx *ui);

#endif
