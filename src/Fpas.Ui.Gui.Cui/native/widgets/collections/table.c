#include "../../fpas_cui.h"

int fpas_cui_table_begin(cui_ctx *ui, const char *id, const char *const *titles,
    const int *kinds, const float *widths, const int *alignments, const int *sortable, int count,
    int height_kind, float height, int striped, int compact, int selectable, int enabled,
    int *sort, int *descending, int focus_header, int *headers) {
    const cui_id key = cui_make_id(ui, id);
    const int node = ui->node_count;
    cui_table_column *cols = cui__frame_alloc(ui, (size_t)count * sizeof *cols);
    for (int index = 0; index < count; ++index) {
        cols[index] = (cui_table_column){
            .title = cui__frame_strdup(ui, titles[index], -1),
            .width = kinds[index] == 1 ? CUI_GROW(widths[index]) : fpas_extent(kinds[index], widths[index]),
            .align = (cui_text_align)alignments[index], .sortable = sortable[index] != 0
        };
    }
    cui_table_sort order = { *sort, *descending != 0 };
    const cui_table_opts options = {
        .sort = *sort >= -1 ? &order : NULL, .height = fpas_extent(height_kind, height),
        .striped = striped != 0, .compact = compact != 0, .selectable = selectable != 0
    };
    if (enabled && focus_header >= 0 && !cui_is_disabled(ui))
        ui->focus_next_request = cui__hash(&focus_header, sizeof focus_header, key ^ 0x4EADu);
    cui_push_disabled(ui, !enabled);
    cui_table_begin_ex(ui, id, cols, count, &options);
    cui_pop_disabled(ui);
    *sort = order.column;
    *descending = order.descending;
    if (headers)
        for (int index = 0; index < count; ++index)
            headers[index] = fpas_find(ui, cui__hash(&index, sizeof index, key ^ 0x4EADu));
    return node;
}

int fpas_cui_table_row(cui_ctx *ui, const char *id, int selected, int enabled, int focus, int *node) {
    struct cui_table_build *table = ui->building.table;
    const int index = table->row_index;
    const cui_id seed = table->id ^ 0x20Bu;
    const cui_id position = cui__hash(&index, sizeof index, seed), key = cui__hash_str(id, seed);
    if (table->selectable && enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = key;

    /* CUI processes rows by position. Keep focus and pointer capture keyed to the model row. */
    const int first_focus = ui->focus_count;
    if (ui->focus_next_request == key) ui->focus_next_request = position;
    if (ui->focus == key) ui->focus = position;
    if (ui->hot == key) ui->hot = position;
    if (ui->active == key) ui->active = position;
    cui_push_disabled(ui, !enabled || !table->selectable);
    const bool clicked = cui_table_row(ui, selected != 0);
    cui_pop_disabled(ui);
    *node = cui__current(ui);
    if (ui->nodes[*node].id == position) ui->nodes[*node].id = key;
    for (int part = first_focus; part < ui->focus_count; ++part)
        if (ui->focus_order[part] == position) ui->focus_order[part] = key;
    if (ui->focus_next_request == position) ui->focus_next_request = key;
    if (ui->focus == position) ui->focus = key;
    if (ui->hot == position) ui->hot = key;
    if (ui->active == position) ui->active = key;
    if (ui->last_id == position) ui->last_id = key;
    cui_push_id(ui, id);
    return clicked;
}

void fpas_cui_table_row_end(cui_ctx *ui) { cui_pop_id(ui); }

void fpas_cui_table_cell(cui_ctx *ui, const char *text) {
    if (text) cui_table_cell_text(ui, text);
    else cui_table_cell(ui);
}

void fpas_cui_table_end(cui_ctx *ui, int node) {
    cui_table_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
