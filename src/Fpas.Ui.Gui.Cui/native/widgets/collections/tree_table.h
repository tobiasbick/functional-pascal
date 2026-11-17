#ifndef FPAS_CUI_TREE_TABLE_H
#define FPAS_CUI_TREE_TABLE_H
/** Draws a literal tree cell. depth is nonnegative; open is -1 for a leaf, otherwise
 * an input/output expansion boolean. enabled gates interaction; part receives the
 * frame-local chevron (leaf root for leaves). Returns nonzero on an enabled toggle. */
CUI_API int fpas_cui_tree_cell(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int depth, int *open, int enabled, int *part);
/** Begins a virtual tree table; pair with end in this frame.
 * Column arrays have columns entries and use data_grid.h's column flag encoding.
 * labels, parents and flags have count entries; each parent is -1 or precedes its child.
 * Node flags use CUI_TREE_* constants; node data and labels are copied into frame storage.
 * state[3] is sort column (-2 disabled, -1 unsorted), descending and cursor (-1 or node index).
 * options: 1 input, 2 guides, 8 request focus. Height uses shared extent encoding;
 * row_height is at least one logical point. node receives the frame-local root.
 * Returns a frame-owned handle used only by the matching end. */
CUI_API void *fpas_cui_tree_table_begin(cui_ctx *ui, const char *id,
    const char *const *titles, const float *widths, const int *column_flags, int columns,
    const char *const *labels, const int *parents, const uint32_t *flags, int count,
    int *state, int height_kind, float height, float row_height, int options, int *node);
/** Advances to the next realized node, returning its model index or -1 at exhaustion.
 * part receives the frame-local row node, or -1. */
CUI_API int fpas_cui_tree_table_row(cui_ctx *ui, int *part);
/** Draws the current row's name cell with literal label and optional icon;
 * returns its frame-local expansion chevron node, or -1. */
CUI_API int fpas_cui_tree_table_name(cui_ctx *ui, const char *label, const char *icon);
/** Ends the tree table and restores its enclosing grid/tree scope.
 * flags[count] and state[3] receive updated model data. events[2] receives activated
 * node index (-1 for none) and load count. load[64] receives that many node indices.
 * Makes the root the last layout target; handle must be this frame's matching begin result. */
CUI_API void fpas_cui_tree_table_end(cui_ctx *ui, void *handle, uint32_t *flags,
    int *state, int *events, int *load);
#endif
