#ifndef FPAS_CUI_REORDERABLE_LIST_H
#define FPAS_CUI_REORDERABLE_LIST_H

CUI_API int fpas_cui_reorder_begin(cui_ctx *ui, const char *id, const int *order, int count, int enabled, int *proposed);
CUI_API int fpas_cui_reorder_item_begin(cui_ctx *ui, int position, int enabled);
CUI_API void fpas_cui_reorder_item_end(cui_ctx *ui, int node);
CUI_API int fpas_cui_reorder_end(cui_ctx *ui, int node);

#endif
