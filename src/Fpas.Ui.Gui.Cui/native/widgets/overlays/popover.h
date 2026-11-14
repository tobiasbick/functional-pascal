#ifndef FPAS_POPOVER_H
#define FPAS_POPOVER_H

/** Begins a controlled CUI popover after its anchor.
 * @param ui Drawing context.
 * @param id Stable native panel identity.
 * @param placement CUI placement.
 * @param width Logical width or zero for intrinsic width.
 * @param padding Logical inset or zero for CUI's default.
 * @param gap Logical spacing or zero for CUI's default.
 * @param open Model-owned visibility.
 * @param enabled Input permission.
 * @param node Receives the open panel node, or -1.
 * @param owner Receives the native state owner, or zero.
 * @return Bit 1 draws the body and bit 2 proposes dismissal. */
CUI_API int fpas_cui_popover_begin(cui_ctx *ui, const char *id, int placement,
    float width, float padding, float gap, int open, int enabled, int *node, unsigned int *owner);

/** Ends the popup and restores the anchor's layout target.
 * @param ui Drawing context.
 * @param base Anchor's frame-local node. */
CUI_API void fpas_cui_popover_end(cui_ctx *ui, int base);

#endif
