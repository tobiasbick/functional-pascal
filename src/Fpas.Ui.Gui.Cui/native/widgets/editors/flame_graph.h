// [Documentation](../../../../../docs/ui/gui/editing-tools/flame-graph.md)
#ifndef FPAS_CUI_FLAME_GRAPH_H
#define FPAS_CUI_FLAME_GRAPH_H
/** Detached flame-graph view and interaction results. */
typedef struct {
    double start, end; /**< Visible time bounds. */
    float scroll; /**< Vertical scroll offset. */
    int selected, zoomed; /**< Selected span and double-click/Enter zoom target, minus one when absent. */
} fpas_flame_view;
/** Draws a flame graph and updates its detached view.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param spans Borrowed ordered spans, used during this call.
 * @param count Number of spans.
 * @param view Input/output time range, scroll, selection and zoom result.
 * @param minimum Minimum zoom range in time units.
 * @param height Logical height; zero selects the native default where supported.
 * @param row_height Logical span row height, or zero for the native default.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param flags Enabled bit one and focus-request bit two.
 * @return Nonzero for an enabled view, selection or zoom proposal.
 */
CUI_API int fpas_cui_flame_graph(cui_ctx *ui, const char *id, const cui_flame_span *spans, int count, fpas_flame_view *view, double minimum, float height, float row_height, int width_kind, float width, int flags);
#endif
