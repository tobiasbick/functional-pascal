// [Documentation](../../../../../docs/ui/gui/instruments/readout.md)
#ifndef FPAS_CUI_READOUT_H
#define FPAS_CUI_READOUT_H

/** Draws a labeled numeric readout with a stable inspection identity.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param label Literal readout caption.
 * @param value Literal formatted value.
 * @param unit Optional literal unit.
 * @param detail Optional literal supporting text.
 * @param size Native readout size ordinal.
 * @param tone Native color tone ordinal.
 * @param accent Whether to use accent styling.
 * @param align Native horizontal alignment.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 */
CUI_API void fpas_cui_readout(cui_ctx *ui, const char *id, const char *label, const char *value,
    const char *unit, const char *detail, int size, int tone, int accent, int align,
    int width_kind, float width);

#endif
