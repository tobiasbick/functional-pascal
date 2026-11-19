// [Documentation](../../../../docs/ui/gui/cui.md#theme-and-resources)
#include "../fpas_cui.h"

CUI_API int fpas_cui_font_load(cui_ctx *ui, const void *data, size_t size) {
    return cui_font_load_memory(ui, data, size, true);
}
CUI_API int fpas_cui_font_role(cui_ctx *ui, int role) { return CUI__USER_FONT_BASE + cui__resolve_font(ui, role); }
CUI_API void fpas_cui_font_set_role(cui_ctx *ui, int role, int font) { cui_font_set_role(ui, role, font); }
CUI_API void fpas_cui_font_fallback(cui_ctx *ui, int font, int fallback) { cui_font_add_fallback(ui, font, fallback); }
CUI_API int fpas_cui_emoji_load(cui_ctx *ui, const void *data, size_t size) {
    return cui_emoji_font_load_memory(ui, data, size, true);
}
CUI_API int fpas_cui_emoji_system(cui_ctx *ui) { return cui_emoji_font_load_system(ui); }
CUI_API int fpas_cui_emoji_supported(cui_ctx *ui, const char *text) { return cui_emoji_supported(ui, text, -1); }
