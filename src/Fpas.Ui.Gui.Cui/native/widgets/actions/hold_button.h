#ifndef FPAS_HOLD_BUTTON_H
#define FPAS_HOLD_BUTTON_H

/** Draws a CUI hold button with a stable identity and literal caption.
 * @param ui Current UI-thread context.
 * @param id Stable native identity with an empty visible label.
 * @param label Literal button caption.
 * @param icon Optional CUI glyph, or NULL.
 * @param duration_ms Required hold duration in milliseconds.
 * @param variant Shared button variant ordinal.
 * @param enabled Whether the action accepts input.
 * @param focus Whether to request keyboard focus.
 * @return Nonzero on the frame that completes the hold. */
CUI_API int fpas_cui_hold_button(cui_ctx *ui, const char *id, const char *label, const char *icon,
                                 int duration_ms, int variant, int enabled, int focus);

#endif
