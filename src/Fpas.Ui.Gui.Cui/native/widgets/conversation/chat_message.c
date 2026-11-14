#include "../../fpas_cui.h"

void fpas_cui_chat_message(cui_ctx *ui, const char *id, const char *text, const char *author,
    const char *time, const char *status, const char *avatar, int flags, int enabled) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool markdown = (flags & 8) != 0;
    const cui_chat_opts options = { .mine = (flags & 1) != 0, .author = author, .time = time,
        .status = status, .grouped = (flags & 2) != 0, .last_in_group = (flags & 4) != 0,
        .markdown = markdown, .avatar = fpas_avatar_image_for(ui, key, avatar)->texture };
    cui_push_disabled(ui, !enabled || markdown);
    cui_chat_message_ex(ui, text, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, first, key);
}
