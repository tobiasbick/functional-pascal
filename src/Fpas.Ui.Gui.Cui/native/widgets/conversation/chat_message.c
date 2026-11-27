// [Documentation](../../../../../docs/ui/gui/social-and-media/chat-message.md)
#include "../../fpas_cui.h"

const char *fpas_cui_chat_message(cui_ctx *ui, const char *id, const char *text, const char *author,
    const char *time, const char *status, const char *avatar, int flags, int enabled,
    int *offset, int *line, int *checked, const char **hovered, int *hover_changed) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool markdown = (flags & 8) != 0;
    const bool report = markdown && (flags & 16) != 0;
    cui_markdown_result event = { .task_offset = -1 };
    const cui_chat_opts options = { .mine = (flags & 1) != 0, .author = author, .time = time,
        .status = status, .grouped = (flags & 2) != 0, .last_in_group = (flags & 4) != 0,
        .markdown = markdown, .markdown_result = report ? &event : NULL,
        .avatar = fpas_avatar_image_for(ui, key, avatar)->texture };
    cui_push_disabled(ui, !enabled || (markdown && !report));
    cui_push_id(ui, id);
    cui_chat_message_ex(ui, markdown ? cui__frame_strdup(ui, text, -1) : text, &options);
    cui_pop_id(ui);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, first, key);
    *offset = event.task_offset;
    *line = event.task_line;
    *checked = event.task_checked;
    *hovered = event.hovered_link;
    const cui_id current = event.hovered_link ? cui__hash_str(event.hovered_link, key) : 0;
    cui_id *previous = cui__state_lazy(ui, key, sizeof *previous, report);
    *hover_changed = report && current != *previous;
    if (previous) *previous = current;
    return event.link;
}
