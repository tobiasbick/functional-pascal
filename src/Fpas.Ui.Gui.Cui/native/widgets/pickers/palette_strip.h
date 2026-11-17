#ifndef FPAS_CUI_PALETTE_STRIP_H
#define FPAS_CUI_PALETTE_STRIP_H
/** Draws a palette well and writes proposed colors and locks.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param label Literal well caption.
 * @param colors One to sixteen packed RGBA colors, red in the low byte; updated in place.
 * @param locked Matching lock flags, updated in place.
 * @param count Number of colors and locks.
 * @param seed Optional model-owned shuffle seed, updated in place.
 * @param display Optional value caption.
 * @param height Logical swatch height, or zero for the native default.
 * @param flags Bit 0 enables input, bit 1 hides Shuffle, bit 3 selects plain styling.
 * @param focus Requested swatch index, or minus one.
 * @param parts Receives count swatch nodes and one Shuffle node; absent nodes are minus one.
 * @return Native action flags when enabled, otherwise zero. */
CUI_API int fpas_cui_palette_strip(cui_ctx *ui, const char *id,
    const char *label, uint32_t *colors, int *locked, int count, uint32_t *seed, const char *display, float height, int flags, int focus, int *parts);
#endif
