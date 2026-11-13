#ifndef FPAS_CUI_VEIL_H
#define FPAS_CUI_VEIL_H
CUI_API int fpas_cui_veil_begin(cui_ctx *ui, const char *id, int revealed, const char *label, const char *hint, float radius, uint32_t color, int width_kind, float width, int flags);
CUI_API void fpas_cui_veil_end(cui_ctx *ui);
#endif
