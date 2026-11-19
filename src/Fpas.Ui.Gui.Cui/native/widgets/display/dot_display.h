// [Documentation](../../../../../docs/ui/gui/instruments/dot-display.md)
#ifndef FPAS_CUI_DOT_DISPLAY_H
#define FPAS_CUI_DOT_DISPLAY_H

/** Draws a dot-matrix display and makes it the last widget.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param text Literal NUL-terminated UTF-8 display text.
 * @param pitch Logical dot pitch.
 * @param palette Native palette index, or minus one for rgba.
 * @param rgba Packed RGBA color when palette is minus one.
 * @param align Native horizontal text alignment.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param flags Accent bit one, no-dim bit two, marquee bit four.
 */
CUI_API void fpas_cui_dot_display(cui_ctx *ui, const char *id, const char *text, float pitch,
    int palette, uint32_t rgba, int align, int width_kind, float width, int flags);

#endif
