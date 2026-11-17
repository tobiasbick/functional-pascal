#ifndef FPAS_CUI_ICON_H
#define FPAS_CUI_ICON_H
/** Draws a native icon.
 * @param ui Current UI-thread frame context.
 * @param glyph NUL-terminated UTF-8 icon glyph.
 * @param size Logical glyph size.
 * @param color Packed RGBA icon color.
 */
CUI_API void fpas_cui_display_icon(cui_ctx *ui, const char *glyph, float size, uint32_t color);
#endif
