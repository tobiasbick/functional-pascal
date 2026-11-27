// [Documentation](../../../../../docs/ui/gui/editing-tools/gradient-editor.md)
#ifndef FPAS_CUI_GRADIENT_EDITOR_H
#define FPAS_CUI_GRADIENT_EDITOR_H
/** Samples a ramp, including empty and unordered stops. Returns packed straight RGBA. */
CUI_API uint32_t fpas_cui_gradient_eval(const cui_gradient_stop *stops, int count, float position);

/** Draws editable color stops and returns a model proposal.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param stops Writable gradient stops with capacity elements; valid through this call.
 * @param count Current element count, updated after insertion or removal.
 * @param capacity Writable element capacity; must be at least the current count.
 * @param selected Selected element index, updated by native interaction; minus one means none.
 * @param added Receives the added element index, or minus one.
 * @param removed Receives the removed element index, or minus one.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param height Logical height; zero selects the native default where supported.
 * @param enabled Whether this widget accepts input.
 * @return Bit one reports a proposal; bit two reports a stop edit.
 */
CUI_API int fpas_cui_gradient_editor(cui_ctx *ui, const char *id, cui_gradient_stop *stops, int *count, int capacity, int *selected, int *added, int *removed, int width_kind, float width, float height, int enabled);
#endif
