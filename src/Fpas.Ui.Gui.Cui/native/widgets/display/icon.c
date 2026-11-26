// [Documentation](../../../../../docs/ui/gui/text-and-display/icon.md)
#include "../../fpas_cui.h"

/** Draws an icon with either a theme token or an explicit RGBA color. */
void fpas_cui_display_icon(cui_ctx *ui, const char *glyph, float size, uint32_t color, int theme_token) {
    if (theme_token == -1 && (color >> 24) == 0) {
        /* Preserve layout when explicit zero alpha would select the foreground in cui_icon. */
        cui__icon_node(ui, glyph, size > 0 ? size : 16, CUI_TRANSPARENT);
        return;
    }
    cui_color ink = theme_token > 0 ? CUI_TOKEN_COLOR(theme_token) : fpas_chart_color(theme_token, color);
    cui_icon(ui, glyph, size, ink);
}
