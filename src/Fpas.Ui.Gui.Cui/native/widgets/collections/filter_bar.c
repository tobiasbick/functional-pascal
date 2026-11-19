// [Documentation](../../../../../docs/ui/gui/data-grid/filter-bar.md)
#include "../../fpas_cui.h"

static void fpas_filter_retire(cui_ctx *ui, cui_id owner) {
    for (int k = 0; k < 32; ++k) {
        const cui_id chip = cui__hash_str("chip", cui__hash(&k, sizeof k, owner));
        cui_popup *popup = cui__popup_get(ui, chip ^ 0x3E40u, false);
        if (popup) popup->open = false;
        fpas_forget_state(ui, chip);
        fpas_forget_state(ui, (chip ^ 0x3E40u) ^ 0x4E4156u);
    }
    fpas_forget_state(ui, (cui__hash_str("Filter##add", owner) ^ 0x3E40u) ^ 0x4E4156u);
}

int fpas_cui_filter_bar(cui_ctx *ui, const char *id, const char *const *names,
    const char *const *icons, const char *const *values, const int *counts, int count,
    uint32_t *selected, char *search, int capacity, int enabled, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_filter_field fields[32];
    for (int k = 0, at = 0; k < count; ++k) {
        fields[k] = (cui_filter_field){ names[k], icons[k][0] ? icons[k] : NULL,
            counts[k] ? values + at : NULL, counts[k] };
        at += counts[k];
    }
    cui_push_id(ui, id);
    const cui_id search_key = cui_make_id(ui, "#search");
    const cui_id add_key = cui_make_id(ui, "Filter##add");
    const cui_id clear_key = cui_make_id(ui, "Clear all");
    if (!enabled) {
        for (int k = 0; k < count; ++k) {
            const cui_id chip = cui__hash_str("chip", cui__hash(&k, sizeof k, key));
            cui_popup *panel = cui__popup_get(ui, chip ^ 0x3E40u, false);
            if (panel) panel->open = false;
        }
    }
    cui_pop_id(ui);
    cui_popup *menu = cui__popup_get(ui, add_key ^ 0x3E40u, false);
    if (!enabled && menu) menu->open = false;
    if (enabled && focus) ui->focus_next_request = search ? search_key : add_key;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_filter_bar(ui, id, fields, count, selected, search, (size_t)capacity);
    cui_pop_disabled(ui);
    fpas_own(ui, key, search_key, add_key, add_key ^ 0x3E40u);
    fpas_on_forget(ui, key, fpas_filter_retire);
    parts[0] = node;
    parts[1] = search ? fpas_find(ui, search_key) : -1;
    parts[2] = fpas_find(ui, add_key);
    parts[3] = fpas_find(ui, clear_key);
    cui_push_id(ui, id);
    for (int k = 0; k < count; ++k) {
        cui_push_id_int(ui, k);
        const cui_id chip = cui_make_id(ui, "chip");
        parts[4 + k] = fpas_find(ui, chip);
        cui_pop_id(ui);
    }
    cui_pop_id(ui);
    ui->last_node = node;
    ui->last_id = key;
    return enabled && changed;
}
