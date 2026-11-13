#ifndef FPAS_CUI_KEYMAP_H
#define FPAS_CUI_KEYMAP_H
CUI_API void fpas_cui_keymap(cui_ctx *ui, const char *id, const cui_keymap_binding * bindings, int bindings_count, const char *const * categories, int categories_count, int modifiers, int hide_function_row, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
