// [Documentation](../../../../../docs/ui/gui/developer-tools/pipeline.md)
#include "../../fpas_cui.h"

void fpas_cui_pipeline(cui_ctx *ui, const char *id, const char *const *stages, int stages_count,
    const cui_pipeline_job *jobs, int jobs_count, int selection, int width_kind, float width,
    int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool selectable = selection >= -1;
    *out = fpas_dev_empty();
    const int focus_index = selection >= 0 ? selection : 0;
    const cui_id group = key ^ 0x70B5u;
    const cui_id focus = focus_index < jobs_count ? cui__hash(&focus_index, sizeof focus_index, group) : 0;
    if (enabled && (flags & 2) && focus && !cui__keys_covered(ui)) ui->focus_next_request = focus;
    const cui_id previously_focused = fpas_dev_focused_item(ui, group, jobs_count);
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && previously_focused && fpas_dev_save_keys(ui, previously_focused, keys);
    const cui_pipeline_opts o = { .width = fpas_extent(width_kind, width), .selected = selectable ? &selection : NULL };
    cui_push_disabled(ui, !enabled);
    const cui_pipeline_result r = cui_pipeline_ex(ui, id, stages, stages_count, jobs, jobs_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    const cui_id focused = fpas_dev_focused_item(ui, group, jobs_count);
    owner->focus = focused ? focused : focus;
    out->index = selectable ? selection : -1;
    out->action = r.clicked;
    out->hover = r.hovered;
}
