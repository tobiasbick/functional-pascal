// [Documentation](../../../../../docs/ui/gui/developer-tools/problems-panel.md)
#include "../../fpas_cui.h"

void fpas_cui_problems(cui_ctx *ui, const char *id, const cui_diagnostic *diagnostics, int diagnostics_count,
    float height, int hide_toolbar, int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id list = key ^ 0xB0B5u, state = key ^ 0x9B0Bu;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id viewport = cui_make_id(ui, "rows");
    cui_pop_id(ui);
    // CUI-136 (docs/future/cui-upstream-bugs.md#cui-136): a disabled list must decline wheel input.
    fpas_document_scroll_input(ui, viewport, enabled);
    *out = fpas_dev_empty();
    if (enabled && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = list;
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && fpas_dev_save_keys(ui, list, keys);
    const cui_problems_opts o = { .width = fpas_extent(width_kind, width), .height = height, .no_toolbar = hide_toolbar };
    cui_push_disabled(ui, !enabled);
    const cui_problems_result r = cui_problems_ex(ui, id, diagnostics, diagnostics_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    const int scroll = fpas_find(ui, viewport);
    if (!enabled && scroll >= 0) ui->nodes[scroll].paint = fpas_disabled_document_scroll_layout;
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    fpas_own(ui, key, state, list, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = list;
    out->index = r.selected;
    out->action = r.activated;
    out->count = r.shown;
}
