// [Documentation](../../../../../docs/ui/gui/files/path-bar.md)
#ifndef FPAS_CUI_PATH_BAR_H
#define FPAS_CUI_PATH_BAR_H

/** Draws a folder path with clickable segments and optional text entry.
 * @param ui Current drawing context.
 * @param id Stable path bar identity.
 * @param segments Ordered path component captions.
 * @param count Number of segments.
 * @param text Optional text used when editing the path.
 * @param icon Optional leading glyph.
 * @param placeholder Optional placeholder for text entry.
 * @param size Native control size.
 * @param width_kind Logical extent kind.
 * @param width Logical extent value.
 * @param flags Bit 1 enables input, bit 2 enables text entry, and bit 4 enables child menus.
 * @param focus Segment index to focus, or -1.
 * @param submitted Receives frame-local submitted text, or NULL.
 * @param editing Receives whether native text editing is active.
 * @param parts Receives two node indices per segment and one for text entry.
 * @param root Receives the path bar root node.
 * @param retained_key Receives the identity of the path bar's retained state.
 * @return Activated segment index, or -1. */
CUI_API int fpas_cui_path_bar(cui_ctx *ui, const char *id, const char *const *segments, int count,
    const char *text, const char *icon, const char *placeholder, int size, int width_kind, float width,
    int flags, int focus, const char **submitted, int *editing, int *parts, int *root, uint32_t *retained_key);

/** Begins an open child menu for the path bar.
 * @param ui Current drawing context.
 * @param id Stable identity shared with the path bar.
 * @return Parent segment index, or -1 when no menu scope was opened. */
CUI_API int fpas_cui_path_menu_begin(cui_ctx *ui, const char *id);

/** Ends a child menu scope.
 * @param ui Current drawing context with a child menu open. */
CUI_API void fpas_cui_path_menu_end(cui_ctx *ui);

/** Draws a child menu item with a literal caption.
 * @param ui Current drawing context with a child menu open.
 * @param label Visible item caption.
 * @param index Item position used as its identity.
 * @return Nonzero when the item was selected. */
CUI_API int fpas_cui_path_menu_item(cui_ctx *ui, const char *label, int index);
#endif
