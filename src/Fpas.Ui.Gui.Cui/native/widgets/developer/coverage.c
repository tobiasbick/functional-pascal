// [Documentation](../../../../../docs/ui/gui/more-developer-tools/coverage.md)
#include "../../fpas_cui.h"

/** Draws the native coverage tree and preserves the selected file when sorting. */
void fpas_cui_coverage(cui_ctx *ui, const char *id, const cui_coverage_file *files, int files_count,
    float height, float low, float high, int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id list = key ^ 0xC0E2u;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id viewport = cui_make_id(ui, "rows");
    cui_pop_id(ui);
    // CUI-136 (docs/future/cui-upstream-bugs.md#cui-136): disabled virtual lists can still scroll.
    fpas_document_scroll_input(ui, viewport, enabled);
    if (enabled && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = list;
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && fpas_dev_save_keys(ui, list, keys);
    *out = fpas_dev_empty();
    const cui_coverage_opts o = { .width = fpas_extent(width_kind, width), .height = height, .low = low, .high = high };
    const cui_id state_key = key ^ 0xC0E1u;
    fpas_own(ui, key, state_key, viewport, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = list;
    /* CUI-142 (docs/future/cui-upstream-bugs.md#cui-142): refresh borrowed paths
       without resetting sorting, folds or selection. */
    cui__cov_state *cache = cui__state_lazy(ui, state_key, sizeof *cache, false);
    int *rows = NULL, selected = -1, sort = -1, folded = -1;
    bool descending = false;
    if (cache && cache->init) {
        cache->init = false;
        cui__cov_build(ui, cache, files, files_count);
        sort = cache->sort;
        descending = cache->desc;
        folded = cache->nfolded;
        /* CUI-144 (docs/future/cui-upstream-bugs.md#cui-144): column clicks
           reorder rows, but CUI stores the selection as a visual row index. */
        if (enabled && ui->mreleased[CUI_MOUSE_LEFT] && cache->sel >= 0 && cache->nn > 0) {
            int count = 0;
            rows = cui__frame_alloc(ui, sizeof *rows * (size_t)cache->nn);
            cui__cov_flatten(ui, cache, 0, rows, &count);
            if (cache->sel < count) selected = rows[cache->sel];
        }
    }
    cui_push_disabled(ui, !enabled);
    const cui_coverage_result result = cui_coverage_ex(ui, id, files, files_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    const int scroll = fpas_find(ui, viewport);
    if (!enabled && scroll >= 0) ui->nodes[scroll].paint = fpas_disabled_document_scroll_layout;

    if (sort >= 0 && (cache->sort != sort || cache->desc != descending)) {
        if (selected >= 0) {
            int count = 0;
            cache->sel = -1;
            cui__cov_flatten(ui, cache, 0, rows, &count);
            for (int i = 0; i < count; ++i) if (rows[i] == selected) { cache->sel = i; break; }
        }
        cui_request_redraw(ui);
    }
    if (folded >= 0 && cache->nfolded != folded) cui_request_redraw(ui);

    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    out->action = result.activated;
    out->x = result.lines;
}
