#ifndef FPAS_ICON_BUTTON_H
#define FPAS_ICON_BUTTON_H

/** Draws an icon button with an identity independent of the glyph and tooltip.
 * @param ui Current UI-thread context.
 * @param id Stable native identity with an empty visible label.
 * @param icon Borrowed CUI glyph.
 * @param tooltip Borrowed accessible name and tooltip.
 * @param shortcut Borrowed shortcut hint, or NULL.
 * @param variant Shared button variant ordinal.
 * @param size CUI control size.
 * @param enabled Whether the button receives focus and input.
 * @param loading Whether the button is busy.
 * @param selected Whether the button shows the selected style.
 * @param focus Whether to request focus.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_icon_button(cui_ctx *ui, const char *id, const char *icon, const char *tooltip,
                                const char *shortcut, int variant, int size, int enabled,
                                int loading, int selected, int focus);

#endif
