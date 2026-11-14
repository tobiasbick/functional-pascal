#ifndef FPAS_CUI_CARD_STACK_H
#define FPAS_CUI_CARD_STACK_H
CUI_API int fpas_cui_card_stack_begin(cui_ctx *ui, const char *id, int count,
    int width_kind, float width, float height, int visible, int enabled, int focus);
CUI_API void fpas_cui_card_stack_item_begin(cui_ctx *ui, int depth);
CUI_API void fpas_cui_card_stack_item_end(cui_ctx *ui);
CUI_API int fpas_cui_card_stack_end(cui_ctx *ui, int root, int enabled, int *top);
#endif
