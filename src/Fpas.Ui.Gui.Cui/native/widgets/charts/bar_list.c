// [Documentation](../../../../../docs/ui/gui/more-charts/bar-list.md)
#include "../../fpas_cui.h"

typedef struct { cui_id key; float y; } fpas_bar_row;
typedef struct { cui_id *keys; fpas_bar_row *rows; int count, painted, capacity; } fpas_bar_items;
static cui_id fpas_bar_binding(cui_id key) { return key ^ 0x42415253u; }

static void fpas_bar_dispose(cui_ctx *ui, void *data) {
    fpas_bar_items *items = data;
    cui__free(ui, items->keys);
    cui__free(ui, items->rows);
}
static void fpas_bar_forget(cui_ctx *ui, cui_id key) {
    fpas_bar_items *items = cui__state_lazy(ui, fpas_bar_binding(key), sizeof *items, false);
    if (items) for (int i = 0; i < items->count; ++i) fpas_forget_state(ui, items->keys[i]);
}

/* CUI-091 (docs/future/cui-upstream-bugs.md#cui-091): use the last painted row. */
static float fpas_bar_pointer_y(cui_ctx *ui, cui_id key, const fpas_bar_items *binding,
    const fpas_viz_item *items, const cui_id *keys, int count, const cui_bar_list_opts *options) {
    if (!binding->painted || ui->hot != key || (ui->active && ui->active != key)) return ui->my;
    const float top = cui__prev_rect(ui, key).y, y = ui->my - top;
    cui_id hit = 0;
    for (int i = binding->painted - 1; i >= 0; --i) {
        const fpas_bar_row row = binding->rows[i];
        const float offset = y - row.y;
        if (offset >= 0 && offset < CUI__BL_ROW) {
            hit = row.key;
            break;
        }
        if (!hit && offset >= -CUI__BL_GAP / 2 && offset < CUI__BL_ROW + CUI__BL_GAP / 2) hit = row.key;
    }

    int index = 0;
    while (index < count && keys[index] != hit) ++index;
    if (index == count) return top - CUI__BL_GAP;
    int rank = index;
    if (!options->keep_order) {
        rank = 0;
        for (int i = 0; i < count; ++i)
            if (items[i].value > items[index].value || (items[i].value == items[index].value && i < index)) ++rank;
    }
    return top + rank * (CUI__BL_ROW + CUI__BL_GAP) + CUI__BL_ROW / 2;
}

int fpas_cui_bar_list(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, const char *format, int maximum_rows, int keep_order,
    int palette, uint32_t rgba, int enabled, int *node) {
    const cui_id key = cui_make_id(ui, id), binding_key = fpas_bar_binding(key);
    fpas_bar_items *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__state_set_dtor(ui, binding_key, fpas_bar_dispose);
    cui_id *keys = cui__frame_alloc(ui, sizeof *keys * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i) keys[i] = cui__hash_str(items[i].label ? items[i].label : "", key ^ 0xB1u);
    const cui_bar_list_opts o = { .width = fpas_extent(width_kind, width), .format = fpas_viz_text(ui, format),
        .max_rows = maximum_rows, .keep_order = keep_order != 0, .color = fpas_chart_color(palette, rgba) };
    const float mouse_y = ui->my, row_y = fpas_bar_pointer_y(ui, key, binding, items, keys, count, &o);
    for (int i = 0; i < binding->count; ++i) {
        bool found = false;
        for (int j = 0; j < count && !found; ++j) found = keys[j] == binding->keys[i];
        if (!found) fpas_forget_state(ui, binding->keys[i]);
    }
    if (binding->capacity < count) {
        cui__free(ui, binding->keys);
        cui__free(ui, binding->rows);
        binding->keys = cui__alloc(ui, sizeof *keys * (size_t)count);
        binding->rows = cui__alloc(ui, sizeof *binding->rows * (size_t)count);
        binding->capacity = count;
    }
    if (count) memcpy(binding->keys, keys, sizeof *keys * (size_t)count);
    binding->count = count;
    fpas_own(ui, key, binding_key, 0, 0); fpas_on_forget(ui, key, fpas_bar_forget);
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    ui->my = row_y;
    const int clicked = cui_bar_list_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    ui->my = mouse_y;
    const cui__bl_paint *paint = ui->nodes[*node].paint_data;
    binding->painted = paint->n;
    for (int i = 0; i < paint->n; ++i)
        binding->rows[i] = (fpas_bar_row){ cui__hash_str(paint->rows[i].label, key ^ 0xB1u), paint->rows[i].y };
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
