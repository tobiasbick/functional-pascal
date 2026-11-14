#ifndef FPAS_COPY_BUTTON_H
#define FPAS_COPY_BUTTON_H

/** Draws CUI's copy button and submits text through its clipboard provider.
 * @param ui Current UI-thread context.
 * @param id Logical identity passed to the native copy button.
 * @param text Borrowed clipboard text.
 * @param enabled Whether the button accepts input.
 * @param focus Whether to request focus.
 * @return Bit 1 for a clipboard request, bit 2 for visible confirmation, bit 4 for provider availability. */
CUI_API int fpas_cui_copy_button(cui_ctx *ui, const char *id, const char *text, int enabled, int focus);

#endif
