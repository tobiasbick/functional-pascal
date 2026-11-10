#ifndef FPAS_SPLIT_BUTTON_H
#define FPAS_SPLIT_BUTTON_H

/** Draws a native split button with literal captions.
 * @param ui Current UI-thread context.
 * @param id Stable native identity.
 * @param label Main caption.
 * @param icon Optional main glyph.
 * @param labels Ordered menu captions.
 * @param icons Ordered menu glyphs, with an empty string for absent glyphs.
 * @param count Number of menu items.
 * @param variant Shared button variant ordinal.
 * @param size CUI control size.
 * @param enabled Whether either part accepts input.
 * @param focus Requested focus: zero for none, one for main, two for chevron.
 * @param nodes Output indices for main, chevron and open menu items, or -1.
 * @param open Reports whether the menu is accepting input.
 * @param highlighted Reports the highlighted item index, or -1.
 * @param popup_key Returns the scoped popup identity for lifetime cleanup.
 * @return -1 for no action, 0 for main, or one plus the selected item index. */
CUI_API int fpas_cui_split_button(cui_ctx *ui, const char *id, const char *label, const char *icon,
                                  const char *const *labels, const char *const *icons, int count,
                                  int variant, int size, int enabled, int focus, int *nodes,
                                  int *open, int *highlighted, unsigned int *popup_key);

/** Closes a removed split button's popup.
 * @param ui Current UI-thread context.
 * @param popup_key Popup identity returned by fpas_cui_split_button. */
CUI_API void fpas_cui_split_button_forget(cui_ctx *ui, unsigned int popup_key);

#endif
