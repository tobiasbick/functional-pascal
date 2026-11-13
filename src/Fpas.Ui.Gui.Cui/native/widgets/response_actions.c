#include "../fpas_cui.h"

int fpas_cui_response_actions(cui_ctx *ui, const char *id, const char *copy_text, int *vote, int flags, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_response_actions_opts opts = { .copy_text = copy_text, .vote = vote,
        .no_regenerate = (flags & 2) != 0, .no_share = (flags & 4) != 0 };
    cui_push_disabled(ui, !enabled);
    const int result = cui_response_actions_ex(ui, id, &opts);
    cui_pop_disabled(ui);
    int child = ui->nodes[root].first;
    const bool shown[] = { copy_text != NULL, !opts.no_regenerate, true, true, !opts.no_share };
    for (int i = 0; i < 5; ++i) {
        parts[i] = shown[i] ? child : -1;
        if (parts[i] >= 0) child = ui->nodes[child].next;
    }
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
