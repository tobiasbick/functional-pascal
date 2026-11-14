#include "../../fpas_cui.h"

const char *fpas_cui_markdown(cui_ctx *ui, const char *id, const char *text, int width_kind, float width, float size, int flags, int *offset, int *line, int *checked, const char **hovered, int *hover_changed) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_markdown_opts options = { .width = fpas_extent(width_kind, width), .size = size,
        .fit_content = width_kind == 0, .compact = (flags & 2) != 0, .streaming = (flags & 4) != 0 };
    cui_push_disabled(ui, !(flags & 1));
    const cui_markdown_result event = cui_markdown_ex(ui, id, cui__frame_strdup(ui, text, -1), &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *offset = event.task_offset;
    *line = event.task_line;
    *checked = event.task_checked;
    *hovered = event.hovered_link;
    const cui_id current = event.hovered_link ? cui__hash_str(event.hovered_link, key) : 0;
    cui_id *previous = cui_state(ui, key, sizeof *previous);
    *hover_changed = current != *previous;
    *previous = current;
    return event.link;
}
