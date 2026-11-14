#ifndef FPAS_BUTTON_H
#define FPAS_BUTTON_H

/** Draws a button with a literal label and a separate stable identity.
 * @param ui Current UI-thread context.
 * @param id Stable CUI identity starting with the triple-hash marker.
 * @param text Literal label.
 * @param variant Shared button style ordinal.
 * @param size CUI control size.
 * @param enabled Whether the control accepts input.
 * @param loading Whether the button is busy.
 * @param selected Whether to show the selected appearance.
 * @param focus Whether to request keyboard focus.
 * @param icon Leading CUI glyph, or NULL.
 * @param icon_right Trailing CUI glyph, or NULL.
 * @param tooltip Hover tooltip, or NULL.
 * @param shortcut Shortcut hint, or NULL.
 * @param fill Whether to use the available width.
 * @param width Fixed logical width, or -1 for intrinsic width.
 * @param rgba Packed foreground override, or zero for the default.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_button(cui_ctx *ui, const char *id, const char *text,
                           int variant, int size, int enabled, int loading, int selected, int focus,
                           const char *icon, const char *icon_right, const char *tooltip, const char *shortcut,
                           int fill, float width, unsigned int rgba);

#endif
