#include "../../fpas_cui.h"

int fpas_cui_breadcrumb(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    const char **captions = cui__frame_alloc(ui, (size_t)count * sizeof *captions);
    for (int index = 0; index < count; ++index)
        captions[index] = cui__frame_printf(ui, "x###%d", index);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const int row = ui->node_count;
    cui_push_id(ui, id);
    if (allowed && focus) ui->focus_next_request = cui_make_id(ui, captions[focus - 1]);
    cui_push_disabled(ui, !enabled);
    const int choice = cui_breadcrumb(ui, captions, count);
    cui_pop_disabled(ui);
    nodes[0] = row;
    for (int index = 0; index < count; ++index) {
        const cui_id key = cui_make_id(ui, captions[index]);
        const bool current = index == count - 1;
        const int node = current ? ui->nodes[row].last : fpas_find(ui, key);
        if (current) ui->nodes[node].id = cui__hash_str("#current", key);
        fpas_text(ui, node, labels[index]);
        nodes[index + 1] = node;
    }
    cui_pop_id(ui);
    ui->nodes[row].id = group;
    if (!enabled) ui->nodes[row].opacity = CUI__DISABLED;
    ui->last_node = row;
    ui->last_id = group;
    return allowed ? choice : -1;
}
