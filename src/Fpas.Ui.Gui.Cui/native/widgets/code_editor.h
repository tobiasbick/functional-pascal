#ifndef FPAS_CUI_CODE_EDITOR_H
#define FPAS_CUI_CODE_EDITOR_H
CUI_API int fpas_cui_code_editor(cui_ctx *ui, const char *id, char *text, size_t capacity, const char *language, const char *placeholder, int width_kind, float width, float height, float size, int tab_size, int flags, int *line, int *column, int *lines);
#endif
