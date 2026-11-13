#ifndef FPAS_CUI_NODE_EDITOR_H
#define FPAS_CUI_NODE_EDITOR_H
CUI_API int fpas_cui_node_editor_begin(cui_ctx *ui, const char *id, cui_node_view view,
    int width_kind, float width, int height_kind, float height, float minimum, float maximum, int flags);
CUI_API void fpas_cui_node_editor_end(cui_ctx *ui, cui_node_view *result);
CUI_API void fpas_cui_node_begin(cui_ctx *ui, int id, float *x, float *y, const char *title, float width, const char *icon);
CUI_API void fpas_cui_node_pin(cui_ctx *ui, int id, const char *label, int output, int type);
CUI_API int fpas_cui_node_event(cui_ctx *ui, int kind, int *first, int *second);
#endif
