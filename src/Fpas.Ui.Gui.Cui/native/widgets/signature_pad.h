#ifndef FPAS_SIGNATURE_PAD_H
#define FPAS_SIGNATURE_PAD_H

/** Draws CUI's signature pad.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param placeholder Optional literal instruction.
 * @param height Logical height, or zero for the default.
 * @param rgba Packed RGBA ink color, or zero for the default.
 * @param enabled Whether to accept drawing and clearing.
 * @param focus_clear Whether to focus the visible Clear button.
 * @param node Receives the pad root.
 * @param clear Receives the Clear button, or -1 when hidden.
 * @return Whether the native signature changed. */
CUI_API int fpas_cui_signature_pad(cui_ctx *ui, const char *id, const char *placeholder, float height,
    uint32_t rgba, int enabled, int focus_clear, int *node, int *clear);
#endif
