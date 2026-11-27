// [Documentation](../../../../../docs/ui/gui/data-grid/tree-table.md)
#ifndef FPAS_CUI_TREE_TABLE_H
#define FPAS_CUI_TREE_TABLE_H

#include "../overlays/menu.h"
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
 * options: 1 input, 2 guides, 4 dragging, 8 request focus, 16 omit the label callback.
 * drop_permissions is NULL or carries a bit per placement for each node. Both extents use the shared encoding;
 * row_height is at least one logical unit, or zero for the default of 32. node receives the frame-local root.
 * Returns a frame-owned handle used only by the matching end. */
CUI_API void *fpas_cui_tree_table_begin(cui_ctx *ui, const char *id,
    const char *const *titles, const float *widths, const int *column_flags, int columns,
    const char *const *labels, const int *parents, const uint32_t *flags, int count,
    int *state, int height_kind, float height, float row_height, int width_kind, float width,
    int options, const uint8_t *drop_permissions, int *node);
/** Advances to the next realized node, returning its model index or -1 at exhaustion.
 * part receives the frame-local row node, or -1. */
CUI_API int fpas_cui_tree_table_row(cui_ctx *ui, int *part);
/** Draws the current row's name cell with literal label and optional icon;
 * returns its frame-local expansion chevron node, or -1. */
CUI_API int fpas_cui_tree_table_name(cui_ctx *ui, const char *label, const char *icon);
/** Ends the tree table and restores its enclosing grid/tree scope.
 * flags[count] and state[3] receive updated model data. events[9] receives activated,
 * load count, expanded, collapsed, selection changed, sort changed, drop count,
 * target and placement. Load and dropped receive the respective node indices.
 * Makes the root the last layout target; handle must be this frame's matching begin result. */
CUI_API void fpas_cui_tree_table_end(cui_ctx *ui, void *handle, uint32_t *flags,
    int *state, int *events, int *load, int *dropped);
/** Asks for table id's context menu after its end, in every frame the table offers one.
 * A right click on a row or Shift+F10 opens it. Returns nonzero while it is open, with
 * the model node it is for in node (otherwise -1). popup and table receive the menu's
 * popup ID and the table root for the matching end in this frame. */
CUI_API int fpas_cui_tree_table_menu_begin(cui_ctx *ui, const char *id, int enabled, int *node,
    unsigned int *popup, int *table);
/** Draws the open menu's rows when visible and ends it. Rows use menu.h's encoding.
 * nodes[count + 1] receives the table root followed by the command nodes, open whether the popup
 * accepts input, highlighted the highlighted entry and owner its lifecycle key.
 * Makes the table root the last layout target again. Returns the chosen entry index, or -1. */
CUI_API int fpas_cui_tree_table_menu_end(cui_ctx *ui, unsigned int popup, int table, int visible,
    const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const fpas_cui_menu_row *rows, int count, int *nodes,
    int *open, int *highlighted, unsigned int *owner);
#endif
