// [Documentation](../../../../../docs/ui/gui/creative-controls/option-action.md)
#ifndef FPAS_CUI_OPTION_ACTION_H
#define FPAS_CUI_OPTION_ACTION_H
/** Draws a segmented choice with an optional action button.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param label Well caption.
 * @param items Literal option captions.
 * @param count Number of options, from one to 32.
 * @param value Current zero-based option; receives the selected option.
 * @param icon Optional action glyph; NULL omits the button.
 * @param tooltip Optional action name.
 * @param display Optional value caption.
 * @param flags Bit 0 enables input, bit 2 shows the action pressed, bit 3 selects plain
 *              styling, and bit 4 enables action activation.
 * @param focus Requested option index, count for the action, or minus one.
 * @param parts Receives option nodes followed by the action node, or minus one.
 * @return The CUI_WELL_CHANGED and CUI_WELL_ACTION bits. */
CUI_API int fpas_cui_option_action(cui_ctx *ui, const char *id,
    const char *label, const char *const *items, int count, int *value, const char *icon, const char *tooltip, const char *display, int flags, int focus, int *parts);
#endif
