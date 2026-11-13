#include "../fpas_cui.h"

typedef struct { cui_id *keys; int count, capacity; } fpas_bar_items;
static cui_id fpas_bar_binding(cui_id key) { return key ^ 0x42415253u; }

static void fpas_bar_dispose(cui_ctx *ui, void *data) {
    fpas_bar_items *items = data; cui__free(ui, items->keys);
}
static void fpas_bar_forget(cui_ctx *ui, cui_id key) {
    fpas_bar_items *items = cui__state_lazy(ui, fpas_bar_binding(key), sizeof *items, false);
    if (items) for (int i = 0; i < items->count; ++i) fpas_forget_state(ui, items->keys[i]);
}

int fpas_cui_bar_list(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, const char *format, int maximum_rows, int keep_order,
    int palette, uint32_t rgba, int enabled, int *node) {
    const cui_id key = cui_make_id(ui, id), binding_key = fpas_bar_binding(key);
    fpas_bar_items *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__state_set_dtor(ui, binding_key, fpas_bar_dispose);
    cui_id *keys = cui__frame_alloc(ui, sizeof *keys * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i) keys[i] = cui__hash_str(items[i].label ? items[i].label : "", key ^ 0xB1u);
    for (int i = 0; i < binding->count; ++i) {
        bool found = false;
        for (int j = 0; j < count && !found; ++j) found = keys[j] == binding->keys[i];
        if (!found) fpas_forget_state(ui, binding->keys[i]);
    }
    if (binding->capacity < count) {
        cui__free(ui, binding->keys); binding->keys = cui__alloc(ui, sizeof *keys * (size_t)count); binding->capacity = count;
    }
    if (count) memcpy(binding->keys, keys, sizeof *keys * (size_t)count);
    binding->count = count;
    fpas_own(ui, key, binding_key, 0, 0); fpas_on_forget(ui, key, fpas_bar_forget);
    const cui_bar_list_opts o = { .width = fpas_extent(width_kind, width), .format = fpas_viz_text(ui, format),
        .max_rows = maximum_rows, .keep_order = keep_order != 0, .color = fpas_chart_color(palette, rgba) };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_bar_list_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
