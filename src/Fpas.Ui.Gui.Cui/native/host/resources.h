#ifndef FPAS_CUI_RESOURCES_H
#define FPAS_CUI_RESOURCES_H
CUI_API void fpas_cui_theme_metrics(cui_ctx *ui, float *values, int write);
CUI_API void *fpas_cui_strings(cui_ctx *ui, void *previous, const char *const *strings);
CUI_API uint32_t fpas_cui_color(int kind, float a, float b, float c, float d, uint32_t left, uint32_t right);
CUI_API int fpas_cui_font_load(cui_ctx *ui, const void *data, size_t size);
CUI_API int fpas_cui_font_role(cui_ctx *ui, int role);
CUI_API void fpas_cui_font_set_role(cui_ctx *ui, int role, int font);
CUI_API void fpas_cui_font_fallback(cui_ctx *ui, int font, int fallback);
CUI_API int fpas_cui_emoji_load(cui_ctx *ui, const void *data, size_t size);
#endif
