// [Documentation](../../../../../docs/ui/gui/developer-tools/problems-panel.md)
#ifndef FPAS_CUI_PROBLEMS_H
#define FPAS_CUI_PROBLEMS_H
/** Draws the virtual list of diagnostics.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param diagnostics Diagnostics in input order.
 * @param diagnostics_count Number of diagnostics.
 * @param height Logical height, zero for the native default, or negative to fill.
 * @param hide_toolbar Nonzero to hide the search and severity controls.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param flags Bit 0 enables input. Bit 1 requests focus for the list.
 * @param out Receives selected and activated diagnostic indices and the filtered count. */
CUI_API void fpas_cui_problems(cui_ctx *ui, const char *id, const cui_diagnostic *diagnostics, int diagnostics_count,
    float height, int hide_toolbar, int width_kind, float width, int flags, fpas_dev_result *out);
#endif
