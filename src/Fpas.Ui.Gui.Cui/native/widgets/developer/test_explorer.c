// [Documentation](../../../../../docs/ui/gui/more-developer-tools/test-explorer.md)
#include "../../fpas_cui.h"

/** Draws the tree and reports its selection, requests and test counts. */
void fpas_cui_test_explorer(cui_ctx *ui, const char *id, const cui_test_item *entries, int entries_count,
    float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id list = key ^ 0x7E50u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id viewport = cui_make_id(ui, "rows");
    cui_pop_id(ui);
    // CUI-136 (docs/future/cui-upstream-bugs.md#cui-136): the virtual list handles wheel input in disabled scopes.
    fpas_document_scroll_input(ui, viewport, allowed);
    *out = fpas_dev_empty();
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = allowed && fpas_dev_save_keys(ui, list, keys);
    const cui_test_explorer_opts o = { .width = fpas_extent(width_kind, width), .height = height };
    cui_push_disabled(ui, !allowed);
    const cui_test_explorer_result r = cui_test_explorer_ex(ui, id, entries, entries_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    const int scroll = fpas_find(ui, viewport);
    if (!allowed && scroll >= 0) ui->nodes[scroll].paint = fpas_disabled_document_scroll_layout;
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    if (width_kind == 0) {
        ui->nodes[root].w = CUI_FIT;
        ui->nodes[root].self_align = CUI__ALIGN_NOSTRETCH;
    }
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    fpas_own(ui, key, key ^ 0x7E51u, viewport, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = list;
    out->index = r.selected;
    out->action = r.activated;
    out->hover = r.run;
    out->flags = r.run_all;
    out->count = r.total;
    out->x = r.passed;
    out->y = r.failed;
    out->z = r.skipped;
}
