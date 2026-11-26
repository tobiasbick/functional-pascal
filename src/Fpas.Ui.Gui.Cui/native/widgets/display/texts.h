// [Documentation](../../../../../docs/ui/gui/text-and-display/text.md)
#ifndef FPAS_CUI_TEXTS_H
#define FPAS_CUI_TEXTS_H
/** Draws literal text using a native semantic role.
 * @param ui Current UI-thread frame context.
 * @param text NUL-terminated UTF-8 text.
 * @param role Body zero, headings one through four, label five, secondary six, muted seven, code eight.
 * @param direction A cui_direction value.
 * @param alignment A cui_text_align value.
 */
CUI_API void fpas_cui_display_text(cui_ctx *ui, const char *text, int role, int direction, int alignment);
#endif
