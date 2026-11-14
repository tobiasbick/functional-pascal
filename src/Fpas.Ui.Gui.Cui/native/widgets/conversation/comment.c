#include "../../fpas_cui.h"

int fpas_cui_comment(cui_ctx *ui, const char *id, const char *author, const char *text,
    const char *time, int depth, const char *badge, int flags, int focus, int *likes, int *liked, int *parts) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 2) && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id keys[] = { cui_make_id(ui, "###like"), cui_make_id(ui, "Reply") };
    cui_pop_id(ui);
    if (enabled && focus >= 0 && focus < 2) ui->focus_next_request = keys[focus];
    bool active = *liked != 0;
    const cui_comment_opts options = { .time = time, .depth = depth, .likes = likes, .liked = &active,
        .edited = (flags & 1) != 0, .badge = badge };
    cui_push_disabled(ui, !enabled);
    const int result = cui_comment_ex(ui, id, author, text, &options);
    cui_pop_disabled(ui);
    *liked = active;
    for (int i = 0; i < 2; ++i) parts[i] = fpas_find(ui, keys[i]);
    fpas_creative_node(ui, first, key);
    return enabled ? result : 0;
}
