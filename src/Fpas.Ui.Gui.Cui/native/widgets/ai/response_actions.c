// [Documentation](../../../../../docs/ui/gui/ai/response-actions.md)
#include "../../fpas_cui.h"

static cui_id response_action_focus(cui_ctx *ui, const char *id, int part) {
    const char *captions[] = { NULL, "Regenerate", "Good response", "Bad response", "Share" };
    const char *icons[] = { NULL, CUI_ICON_REFRESH_CW, CUI_ICON_THUMBS_UP, CUI_ICON_THUMBS_DOWN, CUI_ICON_SHARE_2 };
    cui_push_id(ui, id);
    cui_id key;
    if (part == 0) {
        cui_push_id(ui, "copy");
        key = cui_make_id(ui, "#copy-button");
        cui_pop_id(ui);
    } else {
        key = cui__hash_str(icons[part], cui_make_id(ui, captions[part]));
    }
    cui_pop_id(ui);
    return key;
}

int fpas_cui_response_actions(cui_ctx *ui, const char *id, const char *copy_text, int *vote, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_response_actions_opts opts = { .copy_text = copy_text, .vote = vote,
        .no_regenerate = (flags & 2) != 0, .no_share = (flags & 4) != 0 };
    const bool shown[] = { copy_text != NULL, !opts.no_regenerate, true, true, !opts.no_share };
    if (enabled && focus >= 0 && focus < 5 && shown[focus])
        ui->focus_next_request = response_action_focus(ui, id, focus);
    cui_push_disabled(ui, !enabled);
    const int result = cui_response_actions_ex(ui, id, &opts);
    cui_pop_disabled(ui);
    int child = ui->nodes[root].first;
    for (int i = 0; i < 5; ++i) {
        parts[i] = shown[i] ? child : -1;
        if (parts[i] >= 0) child = ui->nodes[child].next;
    }
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
