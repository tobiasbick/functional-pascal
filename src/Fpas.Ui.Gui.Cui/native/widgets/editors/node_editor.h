// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/node-editor.md)
#ifndef FPAS_CUI_NODE_EDITOR_H
#define FPAS_CUI_NODE_EDITOR_H
/** Begins a node editor and copies the supplied view into frame storage.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param view Input pan and zoom; retrieve the proposal with node_editor_end.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param height_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param height Logical fixed height or percentage, according to height_kind.
 * @param minimum Minimum zoom.
 * @param maximum Maximum zoom.
 * @param flags Enabled bit one, hide-grid bit two, minimap bit four.
 * @return Nonzero when begun; pair a successful begin with exactly one end.
 */
CUI_API int fpas_cui_node_editor_begin(cui_ctx *ui, const char *id, cui_node_view view,
    int width_kind, float width, int height_kind, float height, float minimum, float maximum, int flags);
/** Ends a successfully begun node editor and pops its disabled scope.
 * @param ui Current UI-thread frame context.
 * @param result Receives the detached pan and zoom proposal.
 */
CUI_API void fpas_cui_node_editor_end(cui_ctx *ui, cui_node_view *result);
/** Begins a node inside the current node editor.
 * @param ui Current UI-thread frame context.
 * @param id Application node integer identity.
 * @param x Input/output logical node X coordinate.
 * @param y Input/output logical node Y coordinate.
 * @param title Optional literal title, copied into frame storage.
 * @param width Logical node width.
 * @param icon Optional glyph, copied into frame storage.
 */
CUI_API void fpas_cui_node_begin(cui_ctx *ui, int id, float *x, float *y, const char *title, float width, const char *icon);
/** Draws a pin inside the current node.
 * @param ui Current UI-thread frame context.
 * @param id Application pin integer identity.
 * @param label Optional literal caption, copied into frame storage.
 * @param output Nonzero for an output pin; zero for an input pin.
 * @param type Native pin compatibility type.
 */
CUI_API void fpas_cui_node_pin(cui_ctx *ui, int id, const char *label, int output, int type);
/** Reads a native node-editor proposal.
 * @param ui Current UI-thread frame context.
 * @param kind Zero for link creation, one for link removal, two for node deletion.
 * @param first Receives the first pin, removed link, or deleted node identity.
 * @param second Receives the second pin for link creation; otherwise unspecified.
 * @return Nonzero when the requested event is available.
 */
CUI_API int fpas_cui_node_event(cui_ctx *ui, int kind, int *first, int *second);
#endif
