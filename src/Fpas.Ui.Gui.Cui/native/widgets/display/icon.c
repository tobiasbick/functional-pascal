#include "../../fpas_cui.h"

void fpas_cui_display_icon(cui_ctx *ui, const char *glyph, float size, uint32_t color) {
    cui_icon(ui, glyph, size, fpas_chart_color(-1, color));
}
