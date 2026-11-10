#ifndef FPAS_COPY_BUTTON_H
#define FPAS_COPY_BUTTON_H

/** Draws a stable copy/check icon and submits text to CUI's clipboard provider on activation.
 * @param ui Current UI-thread context.
 * @param id Stable native identity with an empty visible label.
 * @param text Borrowed clipboard text.
 * @param enabled Whether the button accepts input.
 * @param focus Whether to request focus.
 * @param now Monotonic milliseconds for the confirmation deadline.
 * @return Bit 1 for a clipboard request, bit 2 for visible confirmation, bit 4 for provider availability. */
CUI_API int fpas_cui_copy_button(cui_ctx *ui, const char *id, const char *text, int enabled, int focus, int64_t now);

#endif
