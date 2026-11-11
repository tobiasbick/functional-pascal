#ifndef FPAS_CONFIRM_BUTTON_H
#define FPAS_CONFIRM_BUTTON_H

/** Draws CUI's two-step button with separate identity and literal captions.
 * @param ui Current UI-thread context.
 * @param id Stable native identity with an empty visible label.
 * @param label Literal resting caption.
 * @param confirm_label Literal confirmation caption.
 * @param icon Optional resting glyph, or NULL.
 * @param timeout_ms Positive confirmation window in milliseconds.
 * @param variant Shared resting button variant ordinal.
 * @param size Native control size ordinal.
 * @param enabled Whether the control accepts input.
 * @param focus Whether to request keyboard focus.
 * @return Bit 1 on confirmation, bit 2 while armed. */
CUI_API int fpas_cui_confirm_button(cui_ctx *ui, const char *id, const char *label,
    const char *confirm_label, const char *icon, int timeout_ms, int variant, int size, int enabled, int focus);

#endif
