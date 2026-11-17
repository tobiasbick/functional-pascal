#ifndef FPAS_CUI_KEYCAP_H
#define FPAS_CUI_KEYCAP_H
/** Draws native keyboard keycaps.
 * @param ui Current UI-thread frame context.
 * @param keys NUL-terminated native shortcut label.
 */
CUI_API void fpas_cui_display_keycap(cui_ctx *ui, const char *keys);
#endif
