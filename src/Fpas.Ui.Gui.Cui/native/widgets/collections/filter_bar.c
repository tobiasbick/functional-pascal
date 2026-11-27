// [Documentation](../../../../../docs/ui/gui/data-grid/filter-bar.md)
#include "../../fpas_cui.h"

/** Resolves the ID of a native chip within the bar's ID scope. */
static cui_id fpas_filter_chip(cui_id owner, int index) {
    return cui__hash_str("chip", cui__hash(&index, sizeof index, owner));
}

/** Removes retained chip and menu state when the bar leaves the view. */
static void fpas_filter_retire(cui_ctx *ui, cui_id owner) {
    for (int k = 0; k < 32; ++k) {
        const cui_id chip = fpas_filter_chip(owner, k);
        cui_popup *popup = cui__popup_get(ui, chip ^ 0x3E40u, false);
        if (popup) {
            popup->open = false;
            memset(&popup->s, 0, sizeof popup->s);
        }
        fpas_forget_state(ui, chip);
        fpas_forget_state(ui, (chip ^ 0x3E40u) ^ 0x4E4156u);
    }
    fpas_forget_state(ui, (cui__hash_str("Filter##add", owner) ^ 0x3E40u) ^ 0x4E4156u);
}

/** Keeps Add-menu field names literal when CUI interprets a name as a hidden suffix. */
static void fpas_filter_literal_names(cui_ctx *ui, cui_id owner, cui_id popup,
    const char *const *names, int count) {
    const cui_popup *menu = cui__popup_get(ui, popup, false);
    if (!menu || menu->last_frame != ui->frame || (!menu->open && menu->s.x <= 0.01f)) return;
    const cui_id seed = cui__hash(&popup, sizeof popup, owner);
    for (int k = 0; k < count; ++k) {
        const char *name = names[k];
        if (!strstr(name, "##")) continue;
        const char *suffix = strstr(name, "###");
        const cui_id item = cui__hash_str(suffix ? suffix : name, cui__hash(&k, sizeof k, seed));
        const int node = fpas_find(ui, item);
        if (node < 0) continue;
        const int caption = fpas_label_node(ui, node);
        if (caption >= 0) fpas_text(ui, caption, name);
    }
}

/** Draws the native search and filter controls, including a visible focus target when requested. */
int fpas_cui_filter_bar(cui_ctx *ui, const char *id, const char *const *names,
    const char *const *icons, const char *const *values, const int *counts, int count,
    uint32_t *selected, char *search, int capacity, int enabled, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_filter_field fields[32];
    for (int k = 0, at = 0; k < count; ++k) {
        fields[k] = (cui_filter_field){ names[k], icons[k],
            counts[k] ? values + at : NULL, counts[k] };
        at += counts[k];
    }
    cui_push_id(ui, id);
    const cui_id search_key = cui_make_id(ui, "#search");
    const cui_id add_key = cui_make_id(ui, "Filter##add");
    const cui_id menu_key = add_key ^ 0x3E40u;
    const cui_id clear_key = cui_make_id(ui, "Clear all");
    cui_pop_id(ui);
    cui_id requested = 0;
    if (allowed && focus) {
        if (search) requested = search_key;
        else {
            const cui__filter_bar_state *state = cui__state_lazy(ui, key, sizeof *state, false);
            const uint32_t active = state ? state->active : 0;
            if (count > 0) requested = fpas_filter_chip(key, 0);
            for (int k = 0; k < count; ++k) {
                if (!selected[k] && !(active & (1u << k))) { requested = add_key; break; }
            }
        }
    }
    if (!allowed) {
        for (int k = 0; k < count; ++k) {
            const cui_id chip = fpas_filter_chip(key, k);
            cui_popup *panel = cui__popup_get(ui, chip ^ 0x3E40u, false);
            if (panel) panel->open = false;
        }
    }
    cui_popup *menu = cui__popup_get(ui, menu_key, false);
    if (!allowed && menu) menu->open = false;
    if (requested) ui->focus_next_request = requested;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_filter_bar(ui, id, fields, count, selected, search, (size_t)capacity);
    cui_pop_disabled(ui);
    if (requested && ui->focus_next_request == requested) ui->focus_next_request = 0;
    fpas_filter_literal_names(ui, key, menu_key, names, count);
    fpas_own(ui, key, search_key, add_key, menu_key);
    fpas_on_forget(ui, key, fpas_filter_retire);
    parts[0] = node;
    parts[1] = search ? fpas_find(ui, search_key) : -1;
    parts[2] = fpas_find(ui, add_key);
    parts[3] = fpas_find(ui, clear_key);
    for (int k = 0; k < count; ++k) parts[4 + k] = fpas_find(ui, fpas_filter_chip(key, k));
    ui->last_node = node;
    ui->last_id = key;
    return allowed && changed;
}
