// [Documentation](../../../../../docs/ui/gui/editing-tools/curve-editor.md)
#ifndef FPAS_CUI_CURVE_EDITOR_H
#define FPAS_CUI_CURVE_EDITOR_H
/** Borrowed curve presentation settings; flags enable input, snapping and playhead rendering. */
typedef struct {
    const char *x_label, *y_label; /**< Optional literal axis captions, valid during the call. */
    float x_min, x_max, y_min, y_max, height, width, playhead; /**< Axis bounds, logical extents and playhead X. */
    int width_kind, flags; /**< Extent kind and enabled/snap/playhead bits one/two/four. */
    const char *x_format, *y_format; /**< Optional formats for the axis and readout values. */
} fpas_curve_options;
/** Detached edit and pointer results. */
typedef struct {
    int added, removed, changed, hovered; /**< Inserted and removed indices, edit and hover flags. */
    float x, y; /**< Pointer's X coordinate and the curve's value there. */
} fpas_curve_observation;
/** Draws an editable curve and returns a model proposal.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param keys Writable curve keys with capacity elements; valid through this call.
 * @param count Current element count, updated after insertion or removal.
 * @param capacity Writable element capacity; must be at least the current count.
 * @param selected Selected element index, updated by native interaction; minus one means none.
 * @param observation Receives edit indices and pointer results.
 * @param settings Borrowed bounds, layout and flags: enabled one, snap two, playhead four.
 * @return Nonzero for an enabled key edit or selection change.
 */
CUI_API int fpas_cui_curve_editor(cui_ctx *ui, const char *id, cui_curve_key *keys, int *count, int capacity, int *selected, fpas_curve_observation *observation, const fpas_curve_options *settings);
#endif
