// [Documentation](../../../../../docs/ui/gui/developer-tools/network-waterfall.md)
#include "../../fpas_cui.h"

/* [CUI-129](../../../../../docs/future/cui-upstream-bugs.md#cui-129):
 * A timing bar has bounds but no hit entry. The row must keep pointer input. */
static void fpas_waterfall_tip(cui_ctx *ui, cui_id key, int row, const cui_net_request *request) {
    const cui_id bar = cui__hash(&row, sizeof row, key ^ 0x3EF1u) ^ 0xBA2u;
    if (ui->active || !cui__rect_contains(cui__prev_rect(ui, bar), ui->mx, ui->my)) return;

    char timing[192];
    int length = 0;
    for (int phase = 0; phase < 6 && length < (int)sizeof timing - 1; ++phase) {
        if (request->timing[phase] > 0)
            length += snprintf(timing + length, sizeof timing - (size_t)length, "\n%s  %s",
                cui__wf_names[phase], cui__fl_dur(ui, request->timing[phase]));
    }
    if (length < (int)sizeof timing - 1)
        snprintf(timing + length, sizeof timing - (size_t)length, "\nTotal  %s, from %s",
            cui__fl_dur(ui, cui__wf_total(request)), cui__fl_dur(ui, request->start));
    const char *name = request->name ? request->name : "";
    const int name_length = cui__utf8_cut(name, (int)strlen(name), 255 - (int)strlen(timing));
    cui__tooltip_passive(ui, bar, cui__frame_printf(ui, "%.*s%s", name_length, name, timing));
}

void fpas_cui_network_waterfall(cui_ctx *ui, const char *id, const cui_net_request * requests, int requests_count, int selection, float height, double dom_loaded, double loaded, int width_kind, float width, int flags, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id focus = key ^ 0x3EF0u, slot = key ^ 0x3EF2u;
    const bool enabled = fpas_ai_enabled(ui, flags, focus);
    const bool retained = selection == -2;
    const cui__wf_state *state = cui__state_lazy(ui, slot, sizeof *state, false);
    const int previous = retained ? (state ? state->sel : -1) : selection;
    cui_push_id(ui, id);
    const cui_id rows = cui_make_id(ui, "rows");
    cui_pop_id(ui);
    fpas_own(ui, key, rows, slot, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = focus;
    if (!enabled) {
        if (ui->scroll_target == rows) ui->scroll_target = 0;
        fpas_clear_interaction(ui, focus);
        cui_scroll_state *scroll = cui__state_lazy(ui, rows, sizeof *scroll, false);
        if (scroll) scroll->drag_y = false;
    }
    *out = fpas_dev_empty();
    const cui_waterfall_opts o = { .width = fpas_extent(width_kind, width), .height = height,
        .dom_loaded = dom_loaded, .loaded = loaded, .selected = retained ? NULL : &selection };
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && fpas_dev_save_keys(ui, focus, keys);
    cui_push_disabled(ui, !enabled);
    cui_waterfall_result r = cui_waterfall_ex(ui,id,requests,requests_count,&o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof ui->kpressed);
    if (enabled && r.hovered >= 0) fpas_waterfall_tip(ui, key, r.hovered, &requests[r.hovered]);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    state = cui__state_lazy(ui, slot, sizeof *state, false);
    out->index = retained ? state->sel : selection;
    out->flags = out->index != previous;
    out->action = r.activated;
    out->hover = r.hovered;
    out->count = r.clicked;
}
