// [Documentation](../../../../../docs/ui/gui/charts/sparkline.md)
#ifndef FPAS_CUI_SPARKLINE_H
#define FPAS_CUI_SPARKLINE_H
/** Draws a native sparkline and assigns its host inspection identity.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param values Call-scoped sample array.
 * @param count Number of samples.
 * @param width_kind FPAS width extent kind.
 * @param width Width basis for the chosen extent.
 * @param height Logical plot height.
 * @param palette Zero selects Automatic, 1..6 a theme series, minus one explicit RGBA.
 * @param rgba Explicit color with red in the low byte. */
CUI_API void fpas_cui_sparkline(cui_ctx *ui, const char *id, const float *values, int count,
    int width_kind, float width, float height, int palette, uint32_t rgba);
#endif
