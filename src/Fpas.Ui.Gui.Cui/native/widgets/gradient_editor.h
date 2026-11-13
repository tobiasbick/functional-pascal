#ifndef FPAS_CUI_GRADIENT_EDITOR_H
#define FPAS_CUI_GRADIENT_EDITOR_H
CUI_API int fpas_cui_gradient_editor(cui_ctx *ui, const char *id, cui_gradient_stop *stops, int *count, int capacity, int *selected, int *added, int *removed, int width_kind, float width, float height, int enabled);
#endif
