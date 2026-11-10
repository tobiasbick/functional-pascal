/* Shared flex semantics over CUI's measured content and native layout/paint. */
#include "fpas_cui.h"

static float fpas_bound(float value, const fpas_flex *item) {
    return CUI_MAX(item->minimum, CUI_MIN(value, item->maximum < 0 ? 1e20f : item->maximum));
}

static void fpas_allocate(cui_ctx *ui, const fpas_flex **items, float *sizes, int parent, bool horizontal) {
    cui_node *group = &ui->nodes[parent];
    float available = horizontal ? group->rect.w - group->pad[1] - group->pad[3] : group->rect.h - group->pad[0] - group->pad[2];
    float used = 0;
    int count = 0;
    bool found = false;
    for (int c = group->first; c >= 0; c = ui->nodes[c].next) {
        cui_node *node = &ui->nodes[c];
        const fpas_flex *item = items[c];
        sizes[c] = -1;
        if (item && item->horizontal == (int)horizontal) {
            float preferred = horizontal ? node->pref_w : node->pref_h;
            sizes[c] = fpas_bound(item->kind == 2 ? item->basis : preferred, item);
            used += sizes[c]; found = true;
        } else used += horizontal ? node->rect.w : node->rect.h;
        count++;
    }
    if (!found) return;
    if (count > 1) available -= group->gap * (count - 1);
    float remaining = available - used;
    if (!horizontal && (group->flags & CUI_NF_SCROLL_Y || group->paint == fpas_hidden_scroll)) remaining = CUI_MAX(0, remaining);
    for (int pass = 0; pass <= count && fabsf(remaining) > 0.01f; pass++) {
        float weight = 0;
        for (int c = group->first; c >= 0; c = ui->nodes[c].next) {
            if (sizes[c] < 0) continue;
            const fpas_flex *item = items[c];
            bool room = remaining > 0 ? item->maximum < 0 || sizes[c] < item->maximum - 0.01f : sizes[c] > item->minimum + 0.01f;
            if (room) weight += remaining > 0 ? CUI_MAX(item->grow, item->kind == 1 ? 1 : 0) : item->shrink;
        }
        if (weight <= 0) break;
        float change = 0;
        for (int c = group->first; c >= 0; c = ui->nodes[c].next) {
            if (sizes[c] < 0) continue;
            const fpas_flex *item = items[c];
            bool room = remaining > 0 ? item->maximum < 0 || sizes[c] < item->maximum - 0.01f : sizes[c] > item->minimum + 0.01f;
            if (!room) continue;
            float share = remaining > 0 ? CUI_MAX(item->grow, item->kind == 1 ? 1 : 0) : item->shrink;
            float next = fpas_bound(sizes[c] + remaining * share / weight, item);
            change += next - sizes[c]; sizes[c] = next;
        }
        remaining -= change;
        if (fabsf(change) < 0.001f) break;
    }
    for (int c = group->first; c >= 0; c = ui->nodes[c].next) {
        if (sizes[c] < 0) continue;
        cui_node *node = &ui->nodes[c];
        if (horizontal) { node->w = CUI_PX(sizes[c]); node->pref_w = sizes[c]; }
        else { node->h = CUI_PX(sizes[c]); node->pref_h = sizes[c]; }
    }
    if (horizontal) cui__resolve_w(ui, parent); else cui__resolve_h(ui, parent);
}

void fpas_cui_resolve_layout(cui_ctx *ui, const fpas_flex *items, int count) {
    cui__layout(ui);
    if (!count) return;
    const fpas_flex **by_node = cui__frame_alloc(ui, sizeof *by_node * (size_t)ui->node_count);
    float *sizes = cui__frame_alloc(ui, sizeof *sizes * (size_t)ui->node_count);
    memset(by_node, 0, sizeof *by_node * (size_t)ui->node_count);
    for (int i = 0; i < count; i++) by_node[items[i].node] = &items[i];
    for (int axis = 0; axis < 2; axis++) {
        bool horizontal = axis == 0;
        for (int parent = 0; parent < ui->node_count; parent++)
            if (ui->nodes[parent].dir == (horizontal ? CUI_ROW : CUI_COLUMN))
                fpas_allocate(ui, by_node, sizes, parent, horizontal);
        if (horizontal) cui__layout(ui);
    }
}
