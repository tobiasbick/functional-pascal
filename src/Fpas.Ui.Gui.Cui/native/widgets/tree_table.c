#include "../fpas_cui.h"

typedef struct {
    int node, count;
    struct cui_grid_build *previous_grid;
    cui__tt_slot previous_tree;
    cui_tree_node *nodes;
    const char **labels;
    cui_table_sort sort;
    int cursor;
} fpas_tree_frame;

int fpas_cui_tree_cell(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int depth, int *open, int enabled, int *part) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id) ^ 0x7CE1u;
    bool expanded = *open > 0;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_tree_cell(ui, id, depth, *open >= 0, &expanded, icon);
    cui_pop_disabled(ui);
    fpas_text(ui, fpas_label_node(ui, root), label);
    *part = *open < 0 ? root : fpas_find(ui, key);
    if (*open < 0) ui->nodes[root].id = key;
    else *open = expanded;
    ui->last_node = root;
    ui->last_id = ui->nodes[root].id;
    return enabled && changed;
}

static const char *fpas_tree_label(void *user, int node) {
    const fpas_tree_frame *frame = user;
    return frame->labels[node];
}

void *fpas_cui_tree_table_begin(cui_ctx *ui, const char *id,
    const char *const *titles, const float *widths, const int *column_flags, int columns,
    const char *const *labels, const int *parents, const uint32_t *flags, int count,
    int *state, int height_kind, float height, float row_height, int options, int *node) {
    fpas_tree_frame *frame = cui__frame_alloc(ui, sizeof *frame);
    *frame = (fpas_tree_frame){ .node = ui->node_count, .count = count,
        .previous_grid = ui->building.grid, .previous_tree = *cui__tt_slot_get(ui),
        .sort = { state[0], state[1] != 0 }, .cursor = state[2] };
    frame->nodes = cui__frame_alloc(ui, (size_t)(count ? count : 1) * sizeof *frame->nodes);
    frame->labels = cui__frame_alloc(ui, (size_t)(count ? count : 1) * sizeof *frame->labels);
    int *heads = cui__frame_alloc(ui, ((size_t)count + 1) * sizeof *heads);
    for (int k = 0; k <= count; ++k) heads[k] = -1;
    for (int k = count - 1; k >= 0; --k) {
        const int parent = parents[k], group = parent + 1;
        frame->nodes[k] = (cui_tree_node){ parent, heads[k + 1], heads[group], flags[k] };
        heads[group] = k;
        frame->labels[k] = cui__frame_strdup(ui, labels[k], -1);
    }
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0x5C011u, key ^ 0x7EE5u, 0);
    const bool enabled = (options & 1) && !cui_is_disabled(ui);
    if (!enabled) {
        cui__grid_state *grid = cui_state(ui, key, sizeof *grid);
        if (grid->init) grid->resizing = -1;
        cui_scroll_state *scroll = cui_state(ui, key ^ 0x5C011u, sizeof *scroll);
        scroll->drag_x = scroll->drag_y = false;
    }
    fpas_grid_input(ui, key, enabled, options & 8);
    const cui_tree_table_opts settings = {
        .height = fpas_extent(height_kind, height), .row_height = row_height,
        .root = heads[0], .sort = state[0] >= -1 ? &frame->sort : NULL,
        .cursor = &frame->cursor, .guides = (options & 2) != 0,
        .label = fpas_tree_label, .user = frame
    };
    const cui_id focus = ui->focus;
    const float dx = ui->scroll_dx, dy = ui->scroll_dy;
    if (!enabled) ui->scroll_dx = ui->scroll_dy = 0;
    if (!enabled && focus == key) ui->focus = 0;
    cui_push_disabled(ui, !enabled);
    cui_tree_table_begin_ex(ui, id, fpas_grid_columns(ui, titles, widths, column_flags, columns),
        columns, frame->nodes, count, &settings);
    if (!enabled) {
        ui->scroll_dx = dx;
        ui->scroll_dy = dy;
        if (focus == key) ui->focus = focus;
    }
    *node = frame->node;
    return frame;
}

int fpas_cui_tree_table_row(cui_ctx *ui, int *part) {
    const int row = cui_tree_table_row(ui);
    *part = row < 0 ? -1 : cui__current(ui);
    return row;
}

int fpas_cui_tree_table_name(cui_ctx *ui, const char *label, const char *icon) {
    cui__tt_build *tree = cui__tt_current(ui);
    const int n = tree->rows[tree->cur].node;
    cui_tree_table_name(ui, label, icon);
    return fpas_find(ui, cui__hash(&n, sizeof n, tree->id ^ 0xC4E7u));
}

void fpas_cui_tree_table_end(cui_ctx *ui, void *handle, uint32_t *flags,
    int *state, int *events, int *load) {
    fpas_tree_frame *frame = handle;
    const cui_tree_table_events result = cui_tree_table_end(ui);
    for (int k = 0; k < frame->count; ++k) flags[k] = frame->nodes[k].flags;
    state[0] = frame->sort.column;
    state[1] = frame->sort.descending;
    state[2] = frame->cursor;
    events[0] = result.activated;
    events[1] = result.load_count;
    for (int k = 0; k < result.load_count; ++k) load[k] = result.load[k];
    cui_pop_disabled(ui);
    *cui__tt_slot_get(ui) = frame->previous_tree;
    fpas_grid_restore(ui, frame->node, frame->previous_grid);
}
