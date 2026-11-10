#ifndef FPAS_HOST_WIDGETS_H
#define FPAS_HOST_WIDGETS_H

/** One copied canvas operation: rectangle or line, logical coordinates and packed RGBA. */
typedef struct { int kind; float a, b, c, d, thickness; unsigned int color; } fpas_paint;

/** Draws a literal-label widget. Parameters are borrowed for the call.
 * @param ui Owner context.
 * @param id Stable native identity.
 * @param text Literal visible label.
 * @param kind Button=0, checkbox=1, choice=2, tab=3.
 * @param enabled Input permission.
 * @param selected Controlled checked/selected state.
 * @param variant Semantic button role.
 * @param focus Requests focus before drawing.
 * @return Activation bit 1 and checked bit 2. */
CUI_API int fpas_cui_control(cui_ctx *ui, const char *id, const char *text, int kind, int enabled, int selected, int variant, int focus);
/** Opens a clipped layout group; scalar parameters select axis, spacing, alignment and border.
 * @param ui Owner context.
 * @param horizontal Row when nonzero.
 * @param padding Logical inset.
 * @param gap Logical separation.
 * @param align Shared alignment ordinal.
 * @param framed Border permission.
 * @return Frame-local node. */
CUI_API int fpas_cui_group_begin(cui_ctx *ui, int horizontal, float padding, float gap, int align, int framed);
/** Closes a group and restores its inspection identity.
 * @param ui Owner context.
 * @param node Frame-local group. */
CUI_API void fpas_cui_group_end(cui_ctx *ui, int node);
/** Applies node sizing on a parent axis.
 * @param ui Owner context.
 * @param node Frame-local node.
 * @param horizontal Parent axis.
 * @param kind Content=0, fill=1, fixed=2.
 * @param basis Logical fixed basis.
 * @param minimum Lower bound.
 * @param maximum Upper bound, negative for unbounded.
 * @param grow Extra-space weight.
 * @param align Cross-axis placement. */
CUI_API void fpas_cui_node_size(cui_ctx *ui, int node, int horizontal, int kind, float basis, float minimum, float maximum, float grow, int align);
/** Supplies an intrinsic width shared by the cells of a content-sized table column.
 * @param ui Owner context.
 * @param node Current frame node.
 * @param width Nonnegative logical preferred width. */
CUI_API void fpas_cui_preferred_width(cui_ctx *ui, int node, float width);
/** Applies a semantic theme role.
 * @param ui Owner context.
 * @param node Frame-local node.
 * @param role Shared role ordinal. */
CUI_API void fpas_cui_node_role(cui_ctx *ui, int node, int role);
/** Queries focus for a frame-local node.
 * @param ui Owner context.
 * @param node Frame-local node.
 * @return Nonzero when focused. */
CUI_API int fpas_cui_focused(cui_ctx *ui, int node);
/** Queries hover for a frame-local node.
 * @param ui Owner context.
 * @param node Frame-local node.
 * @return Nonzero when hovered. */
CUI_API int fpas_cui_hovered(cui_ctx *ui, int node);
/** Applies editor permission and focus before the paired editor draw/pop-disabled.
 * @param ui Owner context.
 * @param id Editor identity.
 * @param enabled Input permission.
 * @param focus Focus request. */
CUI_API void fpas_cui_editor_options(cui_ctx *ui, const char *id, int enabled, int focus);
/** Reads scroll bounds and optionally applies a controlled offset.
 * @param ui Owner context.
 * @param id Scroll identity.
 * @param mode Read-only=-1, offset=0, end=1.
 * @param offset Requested offset.
 * @param current Receives the native target offset before control reconciliation.
 * @param maximum Receives the current range. */
CUI_API void fpas_cui_scroll_state(cui_ctx *ui, const char *id, int mode, float offset, float *current, float *maximum);
/** Opens or closes an application-controlled modal.
 * @param ui Owner context.
 * @param id Stable modal identity.
 * @param open Model-owned visibility.
 * @return Whether contents must be drawn and ended. */
CUI_API int fpas_cui_modal_begin(cui_ctx *ui, const char *id, int open);
/** Returns the current modal's native identity.
 * @param ui Owner context.
 * @return Modal key while its content is open. */
CUI_API unsigned int fpas_cui_modal_key(cui_ctx *ui);

/** Copies canvas operations into native frame memory.
 * @param ui Owner context.
 * @param id Canvas identity.
 * @param width Logical width.
 * @param height Logical height.
 * @param items Borrowed operations.
 * @param count Number of operations. */
CUI_API void fpas_cui_canvas(cui_ctx *ui, const char *id, float width, float height, const fpas_paint *items, int count);
/** Returns the identity of a frame-local node.
 * @param ui Owner context.
 * @param node Node index.
 * @return Native identity. */
CUI_API unsigned int fpas_cui_node_key(cui_ctx *ui, int node);
/** Releases state owned by a removed control.
 * @param ui Owner context.
 * @param key Previously captured native identity. */
CUI_API void fpas_cui_forget(cui_ctx *ui, unsigned int key);

/** Queries a retained focus identity.
 * @param ui Owner context.
 * @param key Previously captured native key.
 * @return Nonzero when focused. */
CUI_API int fpas_cui_focus_key(cui_ctx *ui, unsigned int key);
/** Distinguishes activation from selection.
 * @param ui Owner context.
 * @return Nonzero for Enter or a double-click. */
CUI_API int fpas_cui_activate(cui_ctx *ui);
/** Starts a host-controlled modal traversal.
 * @param ui Owner context. */
CUI_API void fpas_cui_host_begin(cui_ctx *ui);
/** Begins a selectable row with caller-owned cells.
 * @param ui Owner context.
 * @param id Borrowed native identity.
 * @param enabled Input permission.
 * @param selected Model-owned selection.
 * @param focus Focus request.
 * @param events Receives activation bits.
 * @return Frame-local row node, closed with group_end. */
CUI_API int fpas_cui_choice_row(cui_ctx *ui, const char *id, int enabled, int selected, int focus, int *events);

/** Resolves native layout for current-frame scroll reconciliation.
 * @param ui Owner context. */
CUI_API void fpas_cui_resolve_layout(cui_ctx *ui);
/** Applies a shared palette atomically, without interpolating terminal-invisible states.
 * @param ui Owner context.
 * @param dark Nonzero for dark appearance.
 * @param accent Validated CUI accent family ordinal.
 * @param colors Packed RGBA bytes in semantic token order, excluding NONE. */
CUI_API void fpas_cui_theme(cui_ctx *ui, int dark, int accent, const uint32_t *colors);
/** Copies the active semantic palette, excluding NONE, as packed RGBA bytes.
 * @param ui Owner context.
 * @param colors Receives CUI_TOKEN_COUNT - 1 colors. */
CUI_API void fpas_cui_theme_read(cui_ctx *ui, uint32_t *colors);
/** Begins a vertical viewport.
 * @param ui Owner context.
 * @param id Borrowed native identity.
 * @param scrollbar Whether to paint a scrollbar.
 * @param enabled Input permission.
 * @return Frame-local node, closed with group_end. */
CUI_API int fpas_cui_viewport(cui_ctx *ui, const char *id, int scrollbar, int enabled);
/** Reads a tracked viewport's current offset and range.
 * @param ui Owner context.
 * @param node Tracked viewport node.
 * @param offset Receives its logical offset.
 * @param maximum Receives its logical range. */
CUI_API void fpas_cui_scroll_metrics(cui_ctx *ui, int node, float *offset, float *maximum);
/** Reconciles a controlled scroll after current-frame measurement.
 * @param ui Owner context.
 * @param node Viewport node.
 * @param mode Offset=0 or end=1.
 * @param offset Requested logical offset. */
CUI_API void fpas_cui_scroll_layout(cui_ctx *ui, int node, int mode, float offset);
/** Measures literal table text in the native body font.
 * @param ui Owner context.
 * @param text Borrowed UTF-8 text.
 * @return Logical text width. */
CUI_API float fpas_cui_measure_text(cui_ctx *ui, const char *text);
#endif
