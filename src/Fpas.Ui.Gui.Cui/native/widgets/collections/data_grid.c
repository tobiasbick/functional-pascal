// [Documentation](../../../../../docs/ui/gui/data-grid/data-grid.md)
#include "../../fpas_cui.h"

typedef struct {
    int node;
    struct cui_grid_build *previous;
    bool enabled;
    int rows;
    bool *selected;
    cui_table_sort sort;
    int cursor, anchor;
} fpas_grid_frame;

typedef struct {
    cui_id row, current;
    int capacity, double_clicked;
    bool seen;
    char draft[256];
} fpas_grid_row_state;

static cui_grid_column *fpas_grid_columns(cui_ctx *ui, const char *const *titles,
    const float *widths, const int *flags, int count) {
    cui_grid_column *columns = cui__frame_alloc(ui, (size_t)count * sizeof *columns);
    for (int k = 0; k < count; ++k) {
        columns[k] = (cui_grid_column){
            cui__frame_strdup(ui, titles[k], -1), widths[k],
            (cui_text_align)(flags[k] & 7),
            (flags[k] & 8) != 0, (flags[k] & 16) != 0, flags[k] >> 5
        };
    }
    return columns;
}

static void fpas_grid_restore(cui_ctx *ui, int node, struct cui_grid_build *previous) {
    ui->building.grid = previous;
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

/** Releases one draft's field state so its undo history cannot reach the next draft.
 * CUI-060 (docs/future/cui-upstream-bugs.md#cui-060). */
static void fpas_grid_release_editor(cui_ctx *ui, cui_id id, int row, int column) {
    const uint32_t key[3] = { (uint32_t)row, (uint32_t)column, 0xED17u };
    const cui_id field = cui__hash(key, sizeof key, id) ^ 0xF1Du;
    if (ui->focus == field) ui->focus = id;
    if (ui->tf_select_all == field) ui->tf_select_all = 0;
    fpas_forget_state(ui, field);
}

/** Ends an edit whose original cell is unavailable. */
static void fpas_grid_cancel_edit(cui_ctx *ui, cui_id id, cui__grid_state *state) {
    if (state->edit_row < 0) return;
    fpas_grid_release_editor(ui, id, state->edit_row, state->edit_col);
    state->edit_row = state->edit_col = -1;
}

/** Cancels the active draft when the GUI host retires its grid. */
static void fpas_grid_retire_editor(cui_ctx *ui, cui_id id) {
    cui__grid_state *state = cui__state_lazy(ui, id, sizeof *state, false);
    if (state) fpas_grid_cancel_edit(ui, id, state);
}

static void fpas_grid_input(cui_ctx *ui, cui_id id, int enabled, int focus) {
    if (enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = id;
}

static void fpas_grid_reveal_cursor(cui_scroll_state *scroll, int cursor, float row_height) {
    const float top = (float)cursor * row_height;
    const float bottom = top + CUI__GRID_HEAD + row_height;
    if (scroll->view_h <= 0 || top < scroll->ty) scroll->ty = top;
    else if (bottom > scroll->ty + scroll->view_h) scroll->ty = bottom - scroll->view_h;
}

void *fpas_cui_data_grid_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const float *widths, const int *flags, int columns, int rows, uint8_t *selected,
    int *state, int height_kind, float height, float row_height, int width_kind, float width, int options,
    int *range, int *headers) {
    fpas_grid_frame *frame = cui__frame_alloc(ui, sizeof *frame);
    const cui_id key = cui_make_id(ui, id), edit_key = key ^ 0xF0D617u;
    *frame = (fpas_grid_frame){ .node = ui->node_count, .previous = ui->building.grid,
        .enabled = (options & 1) != 0 && !cui_is_disabled(ui), .rows = rows,
        .sort = { state[0], state[1] != 0 }, .cursor = state[2], .anchor = state[3] };
    if (selected) {
        frame->selected = cui__frame_alloc(ui, (size_t)(rows ? rows : 1) * sizeof(bool));
        for (int k = 0; k < rows; ++k) frame->selected[k] = selected[k] != 0;
    }
    cui__grid_state *retained = cui_state(ui, key, sizeof *retained);
    const cui_id scroll_key = key ^ CUI__GRID_SCROLL;
    cui_scroll_state *scroll = cui_state(ui, scroll_key, sizeof *scroll);
    /* CUI-057: the native cursor reveal requires a previously defined cursor.
       A controlled scroll target takes precedence on the first realization.
       See docs/future/cui-upstream-bugs.md#cui-057. */
    if (!(options & 16) && frame->cursor >= 0 && (!retained->init || retained->cursor < 0))
        fpas_grid_reveal_cursor(scroll, frame->cursor, row_height > 0 ? row_height : 40);
    fpas_grid_row_state *edit = cui_state(ui, edit_key, sizeof *edit);
    edit->double_clicked = -1;
    if (retained->init) {
        /* CUI-062: appended columns need widths without resetting existing resize state.
           See docs/future/cui-upstream-bugs.md#cui-062. */
        for (int column = 0; column < columns; ++column)
            if (retained->widths[column] <= 0)
                retained->widths[column] = widths[column] > 0 ? widths[column] : 150;
        if (frame->cursor < 0) retained->cursor = -1;
        retained->anchor = frame->anchor;
        if (retained->edit_row >= rows || !frame->enabled) fpas_grid_cancel_edit(ui, key, retained);
        if (retained->edit_row >= 0) {
            edit->seen = false;
        }
        if (!frame->enabled) retained->resizing = -1;
    }
    fpas_own(ui, key, scroll_key, edit_key, 0);
    fpas_on_forget(ui, key, fpas_grid_retire_editor);
    if (!frame->enabled) scroll->drag_x = scroll->drag_y = false;
    fpas_grid_input(ui, key, frame->enabled, options & 8);
    const cui_grid_opts settings = {
        .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height), .row_height = row_height,
        .selected = frame->selected, .sort = frame->sort.column >= -1 ? &frame->sort : NULL,
        .striped = (options & 2) != 0, .hide_checkboxes = (options & 4) != 0,
        .cursor = &frame->cursor, .anchor = &frame->anchor
    };
    const cui_id focus = ui->focus;
    const float dx = ui->scroll_dx, dy = ui->scroll_dy;
    if (!frame->enabled) ui->scroll_dx = ui->scroll_dy = 0;
    if (!frame->enabled && focus == key) ui->focus = 0;
    cui_push_disabled(ui, !frame->enabled);
    cui_data_grid_begin_ex(ui, id, fpas_grid_columns(ui, titles, widths, flags, columns),
        columns, rows, &range[0], &range[1], &settings);
    if (!frame->enabled) {
        ui->scroll_dx = dx;
        ui->scroll_dy = dy;
        if (focus == key) ui->focus = focus;
    }
    for (int k = 0; k < columns; ++k)
        headers[k] = fpas_find(ui, cui__hash(&k, sizeof k, key ^ 0x4E1u));
    range[2] = frame->node;
    return frame;
}

int fpas_cui_data_grid_row(cui_ctx *ui, int row, const char *id, int *row_clicked) {
    struct cui_grid_build *grid = ui->building.grid;
    const bool clicked = cui_data_grid_row(ui, row);
    if (row_clicked) *row_clicked = clicked ? 1 : 0;
    const int node = cui__current(ui);
    fpas_grid_row_state *edit = cui_state(ui, grid->id ^ 0xF0D617u, sizeof *edit);
    /* CUI reports row_double on press and release. Deliver each completed pair once. */
    if (clicked && grid->row_double && ui->click_count % 2 == 0 && grid->st->edit_row != row)
        edit->double_clicked = row;
    const cui_id row_key = cui__hash_str(id, grid->id);
    if (grid->st->edit_row == row && edit->row != row_key)
        fpas_grid_cancel_edit(ui, grid->id, grid->st);
    edit->current = row_key;
    if (ui->id_depth) ui->id_stack[ui->id_depth - 1] = row_key;
    return node;
}

void fpas_cui_data_grid_cell(cui_ctx *ui, const char *text) {
    if (text) cui_data_grid_cell_text(ui, text);
    else cui_data_grid_cell(ui);
}

/** Draws the current cell and copies a draft back only on commit. */
int fpas_cui_data_grid_edit(cui_ctx *ui, char *value, int capacity) {
    struct cui_grid_build *grid = ui->building.grid;
    if (!grid || !grid->in_row || grid->col <= 0 || grid->col > grid->ncols) return 0;
    cui__grid_state *state = grid->st;
    const int column = grid->col - 1;
    bool active = state->edit_row == grid->row && state->edit_col == column;
    /* CUI-059 (docs/future/cui-upstream-bugs.md#cui-059):
       Hidden columns must not start or keep the row's keyboard editor. */
    if (grid->hidden[column]) {
        if (active) fpas_grid_cancel_edit(ui, grid->id, state);
        return 0;
    }
    fpas_grid_row_state *edit = cui_state(ui, grid->id ^ 0xF0D617u, sizeof *edit);
    if (active && capacity != edit->capacity) {
        fpas_grid_cancel_edit(ui, grid->id, state);
        active = false;
    }
    char *buffer = active ? edit->draft : value;
    const bool committed = cui_data_grid_edit(ui, buffer, (size_t)capacity);
    if (state->edit_row == grid->row && state->edit_col == column) {
        if (edit->double_clicked == grid->row) edit->double_clicked = -1;
        edit->seen = true;
        if (!active) {
            edit->row = edit->current;
            edit->capacity = capacity;
            memcpy(edit->draft, value, (size_t)capacity);
        }
    } else if (active) fpas_grid_release_editor(ui, grid->id, grid->row, column);
    if (committed) memcpy(value, edit->draft, (size_t)capacity);
    return committed;
}

int fpas_cui_data_grid_end(cui_ctx *ui, void *handle, uint8_t *selected, int *state) {
    fpas_grid_frame *frame = handle;
    const struct cui_grid_build *grid = ui->building.grid;
    const fpas_grid_row_state *edit = cui_state(ui, grid->id ^ 0xF0D617u, sizeof *edit);
    const int double_clicked = edit->double_clicked;
    if (grid->st->edit_row >= 0 && !edit->seen) fpas_grid_cancel_edit(ui, grid->id, grid->st);
    cui_data_grid_end(ui);
    cui_pop_disabled(ui);
    if (selected) for (int k = 0; k < frame->rows; ++k) selected[k] = frame->selected[k];
    state[0] = frame->sort.column;
    state[1] = frame->sort.descending;
    state[2] = grid->st->cursor;
    /* CUI-056: the checkbox updates the retained range origin during row drawing.
       See docs/future/cui-upstream-bugs.md#cui-056. */
    state[3] = grid->st->anchor;
    fpas_grid_restore(ui, frame->node, frame->previous);
    return double_clicked;
}
