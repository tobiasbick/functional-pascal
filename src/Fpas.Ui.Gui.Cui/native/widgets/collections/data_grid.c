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

typedef struct { cui_id row, current; int capacity; char draft[256]; } fpas_grid_edit;

static cui_grid_column *fpas_grid_columns(cui_ctx *ui, const char *const *titles,
    const float *widths, const int *flags, int count) {
    cui_grid_column *columns = cui__frame_alloc(ui, (size_t)count * sizeof *columns);
    for (int k = 0; k < count; ++k) {
        const int align = flags[k] & 7;
        columns[k] = (cui_grid_column){
            cui__frame_strdup(ui, titles[k], -1), widths[k],
            align == 0 ? CUI_TEXT_LEFT : (cui_text_align)align,
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

static void fpas_grid_cancel_edit(cui_ctx *ui, cui_id id, cui__grid_state *state) {
    if (state->edit_row < 0) return;
    uint32_t key[3] = { (uint32_t)state->edit_row, (uint32_t)state->edit_col, 0xED17u };
    const cui_id field = cui__hash(key, sizeof key, id) ^ 0xF1Du;
    if (ui->focus == field) ui->focus = id;
    state->edit_row = state->edit_col = -1;
}

static void fpas_grid_input(cui_ctx *ui, cui_id id, int enabled, int focus) {
    if (enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = id;
}

static void fpas_grid_scroll(cui_ctx *ui, cui_id key, int rows, float row_height, bool enabled) {
    /* CUI-025 (docs/future/cui-upstream-bugs.md#cui-025): clamp before the
       native grid computes its visible row range from the old scroll offset. */
    cui_scroll_state *scroll = cui_state(ui, key ^ 0x5C011u, sizeof *scroll);
    const float content = CUI__GRID_HEAD + (rows ? rows * row_height : 0);
    if (content < scroll->content_h) {
        const float maximum = CUI_MAX(0, content - scroll->view_h);
        scroll->y = cui__clampf(scroll->y, 0, maximum);
        scroll->ty = cui__clampf(scroll->ty, 0, maximum);
        scroll->content_h = content;
    }
    if (!enabled) scroll->drag_x = scroll->drag_y = false;
}

void *fpas_cui_data_grid_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const float *widths, const int *flags, int columns, int rows, uint8_t *selected,
    int *state, int height_kind, float height, float row_height, int options,
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
    if (retained->init) {
        if (frame->cursor < 0) retained->cursor = -1;
        retained->anchor = frame->anchor;
        if (retained->edit_row >= rows || !frame->enabled) fpas_grid_cancel_edit(ui, key, retained);
        if (!frame->enabled) retained->resizing = -1;
    }
    fpas_own(ui, key, key ^ 0x5C011u, edit_key, 0);
    fpas_grid_scroll(ui, key, rows, row_height, frame->enabled);
    fpas_grid_input(ui, key, frame->enabled, options & 8);
    const cui_grid_opts settings = {
        .height = fpas_extent(height_kind, height), .row_height = row_height,
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

int fpas_cui_data_grid_row(cui_ctx *ui, int row, const char *id) {
    struct cui_grid_build *grid = ui->building.grid;
    cui_data_grid_row(ui, row);
    const int node = cui__current(ui);
    fpas_grid_edit *edit = cui_state(ui, grid->id ^ 0xF0D617u, sizeof *edit);
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

int fpas_cui_data_grid_edit(cui_ctx *ui, char *value, int capacity) {
    struct cui_grid_build *grid = ui->building.grid;
    if (!grid || grid->col <= 0) return 0;
    cui__grid_state *state = grid->st;
    fpas_grid_edit *edit = cui_state(ui, grid->id ^ 0xF0D617u, sizeof *edit);
    bool active = state->edit_row == grid->row && state->edit_col == grid->col - 1;
    if (active && capacity != edit->capacity) {
        fpas_grid_cancel_edit(ui, grid->id, state);
        active = false;
    }
    char *buffer = active ? edit->draft : value;
    const bool committed = cui_data_grid_edit(ui, buffer, (size_t)capacity);
    if (!active && state->edit_row == grid->row && state->edit_col == grid->col - 1) {
        edit->row = edit->current;
        edit->capacity = capacity;
        memcpy(edit->draft, value, (size_t)capacity);
    }
    if (committed) memcpy(value, edit->draft, (size_t)capacity);
    return committed;
}

void fpas_cui_data_grid_end(cui_ctx *ui, void *handle, uint8_t *selected, int *state) {
    fpas_grid_frame *frame = handle;
    const struct cui_grid_build *grid = ui->building.grid;
    if (grid->st->edit_row >= 0 &&
        (grid->st->edit_row < grid->first || grid->st->edit_row >= grid->last))
        fpas_grid_cancel_edit(ui, grid->id, grid->st);
    cui_data_grid_end(ui);
    cui_pop_disabled(ui);
    if (selected) for (int k = 0; k < frame->rows; ++k) selected[k] = frame->selected[k];
    state[0] = frame->sort.column;
    state[1] = frame->sort.descending;
    state[2] = frame->cursor;
    state[3] = frame->anchor;
    fpas_grid_restore(ui, frame->node, frame->previous);
}
