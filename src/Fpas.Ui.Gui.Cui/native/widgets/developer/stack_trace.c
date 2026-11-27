// [Documentation](../../../../../docs/ui/gui/more-developer-tools/stack-trace.md)
#include "../../fpas_cui.h"

/** Draws stack frames with frame-owned source previews and reports activation. */
void fpas_cui_stack_trace(cui_ctx *ui, const char *id, const cui_stack_frame *frames, int frames_count,
    const char *title, const char *message, const char *language, int width_kind, float width,
    int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    *out = fpas_dev_empty();
    /* The source preview reads the frame and its source lines during painting. */
    if (!frames || frames_count < 0) frames_count = 0;
    cui_stack_frame *copy = frames_count ? cui__frame_alloc(ui, sizeof *copy * (size_t)frames_count) : NULL;
    for (int i = 0; i < frames_count; ++i) {
        copy[i] = frames[i];
        if (!frames[i].source || frames[i].source_count <= 0) continue;
        const char **source = cui__frame_alloc(ui, sizeof *source * (size_t)frames[i].source_count);
        for (int line = 0; line < frames[i].source_count; ++line)
            source[line] = frames[i].source[line] ? cui__frame_strdup(ui, frames[i].source[line], -1) : NULL;
        copy[i].source = source;
    }
    const cui_stack_trace_opts o = {
        .width = fpas_extent(width_kind, width), .title = title, .message = message,
        .lang = language ? cui__frame_strdup(ui, language, -1) : NULL };
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = allowed && fpas_dev_save_keys(ui, ui->focus, keys);
    cui_push_disabled(ui, !allowed);
    const cui_stack_trace_result r = cui_stack_trace_ex(ui, id, copy, frames_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    if (width_kind == 0) {
        ui->nodes[root].w = CUI_FIT;
        ui->nodes[root].self_align = CUI__ALIGN_NOSTRETCH;
    }
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    cui_push_id(ui, id);
    const cui_id copy_state = cui_make_id(ui, "copy") ^ 0xC0B7u;
    cui_pop_id(ui);
    fpas_own(ui, key, key ^ 0x57ACu, copy_state, 0);
    out->action = r.clicked;
}
