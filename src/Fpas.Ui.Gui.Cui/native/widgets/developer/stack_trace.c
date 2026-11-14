#include "../../fpas_cui.h"

void fpas_cui_stack_trace(cui_ctx *ui, const char *id, const cui_stack_frame * frames, int frames_count, const char * title, const char * message, const char * language, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    /* The source preview reads the frame and its source lines during painting. */
    cui_stack_frame *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)CUI_MAX(1, frames_count));
    for (int i = 0; i < frames_count; ++i) {
        copy[i] = frames[i];
        if (frames[i].source_count <= 0) continue;
        const char **source = cui__frame_alloc(ui, sizeof *source * (size_t)frames[i].source_count);
        for (int line = 0; line < frames[i].source_count; ++line)
            source[line] = frames[i].source[line] ? cui__frame_strdup(ui, frames[i].source[line], -1) : NULL;
        copy[i].source = source;
    }
    const cui_stack_trace_opts o = { .width = fpas_extent(width_kind, width), .title = title, .message = message,
        .lang = language ? cui__frame_strdup(ui, language, -1) : NULL };
    cui_push_disabled(ui, !enabled);
    cui_stack_trace_result r = cui_stack_trace_ex(ui,id,copy,frames_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->action=r.clicked;
}
