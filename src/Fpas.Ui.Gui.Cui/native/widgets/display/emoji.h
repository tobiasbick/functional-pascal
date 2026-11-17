#ifndef FPAS_CUI_EMOJI_H
#define FPAS_CUI_EMOJI_H
/** Draws a native emoji.
 * @param ui Current UI-thread frame context.
 * @param glyph NUL-terminated UTF-8 emoji glyph.
 * @param size Logical glyph size.
 */
CUI_API void fpas_cui_display_emoji(cui_ctx *ui, const char *glyph, float size);
#endif
