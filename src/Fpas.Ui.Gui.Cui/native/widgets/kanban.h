#ifndef FPAS_CUI_KANBAN_H
#define FPAS_CUI_KANBAN_H
CUI_API int fpas_cui_kanban_begin(cui_ctx *ui, const char *id, int enabled);
CUI_API int fpas_cui_kanban_column_begin(cui_ctx *ui, const char *title, int count, int palette, uint32_t color, int limit, float width, int *parts);
CUI_API int fpas_cui_kanban_card_begin(cui_ctx *ui, int key, int focus, int *node);
CUI_API void fpas_cui_kanban_card_end(cui_ctx *ui);
CUI_API void fpas_cui_kanban_column_end(cui_ctx *ui);
CUI_API int fpas_cui_kanban_end(cui_ctx *ui, int root, cui_kanban_move *move);
#endif
