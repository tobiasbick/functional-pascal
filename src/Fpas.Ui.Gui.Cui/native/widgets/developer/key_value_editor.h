#ifndef FPAS_CUI_KEY_VALUE_EDITOR_H
#define FPAS_CUI_KEY_VALUE_EDITOR_H
CUI_API void fpas_cui_key_value_editor(cui_ctx *ui, const char *id, cui_kv_pair *pairs, int *count, int capacity, const char *key_label, const char *value_label, int toggles, int secrets, int read_only, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
