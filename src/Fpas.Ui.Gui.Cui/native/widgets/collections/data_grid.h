// [Documentation](../../../../../docs/ui/gui/data-grid/data-grid.md)
#ifndef FPAS_CUI_DATA_GRID_H
#define FPAS_CUI_DATA_GRID_H
/** Begins a virtual grid; pair with end in this frame, building rows in [range[0], range[1]).
 * titles, widths and flags contain columns entries (1..32); columns are copied to frame storage.
 * Column flags: low three bits are a cui_text_align value (start, center, right, left, end).
 * Bit value 8 is sortable, 16 is fixed, and remaining bits encode priority (0..255 shifted left five).
 * rows is nonnegative; selected is NULL or rows input bytes, copied for the frame.
 * state[4] is sort column (-2 disables sorting, -1 unsorted), descending, cursor and anchor
 * (cursor/anchor are -1 or valid row indices). State is copied back by end.
 * options: 1 input, 2 stripes, 4 hide checkboxes, 8 request grid focus.
 * height_kind/height use the shared extent encoding; row_height is at least one point.
 * range[3] receives first row, exclusive last row and frame-local root node;
 * headers[columns] receives frame-local header nodes, or -1.
 * Returns a frame-owned handle used only by the matching end. */
CUI_API void *fpas_cui_data_grid_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const float *widths, const int *flags, int columns, int rows, uint8_t *selected,
    int *state, int height_kind, float height, float row_height, int options,
    int *range, int *headers);
/** Opens the next realized row in ascending display order; id is its stable model identity.
 * Returns its frame-local node. A changed identity cancels an existing draft at that position. */
CUI_API int fpas_cui_data_grid_row(cui_ctx *ui, int row, const char *id);
/** Advances the current row to a cell; text is literal text, or NULL for composed content. */
CUI_API void fpas_cui_data_grid_cell(cui_ctx *ui, const char *text);
/** Draws editable text in the current cell. value contains a terminated UTF-8 model value;
 * capacity includes the terminator and is 2..256 bytes. Returns nonzero only on commit,
 * copying the draft into value then. Model input is otherwise preserved; changing capacity
 * cancels a draft. The caller may discard a committed proposal to reject it. */
CUI_API int fpas_cui_data_grid_edit(cui_ctx *ui, char *value, int capacity);
/** Ends the matching grid, cancels any draft whose editable cell was omitted and copies outputs.
 * selected is NULL or rows writable bytes; state has four writable entries as above.
 * Restores the enclosing grid and makes the grid root the last layout target.
 * Returns the double-clicked display row, or -1. Cell controls and editors own their input. */
CUI_API int fpas_cui_data_grid_end(cui_ctx *ui, void *handle, uint8_t *selected, int *state);
#endif
