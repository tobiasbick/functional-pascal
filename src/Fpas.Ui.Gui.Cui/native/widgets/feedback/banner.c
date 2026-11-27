// [Documentation](../../../../../docs/ui/gui/disclosure/banner.md)
#include "../../fpas_cui.h"

typedef struct { cui__underline_data underline; float focus; } fpas_banner_action_paint;

static void fpas_paint_banner_action(cui_ctx *ui, cui_node *node, void *data) {
    fpas_banner_action_paint *paint = data;
    cui__paint_underline(ui, node, &paint->underline);
    cui__draw_focus_ring(ui, node->rect, node->radius, paint->focus);
}

static void fpas_banner_action_focus(cui_ctx *ui, int node, cui_id key) {
    cui_node *action = &ui->nodes[node];
    const cui__btn_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (!state || state->focus <= 0.01f || action->paint != cui__paint_underline) return;
    /* CUI-052: link-style banner actions track focus but only paint the hover underline.
       See docs/future/cui-upstream-bugs.md#cui-052. */
    fpas_banner_action_paint *paint = cui__frame_alloc(ui, sizeof *paint);
    *paint = (fpas_banner_action_paint){*(const cui__underline_data *)action->paint_data, state->focus};
    action->paint = fpas_paint_banner_action;
    action->paint_data = paint;
}

int fpas_cui_banner(cui_ctx *ui, const char *id, const char *text, int shown,
    const char *action, const char *icon, int tone, int dismissible, int enabled, int focus,
    int *node, int *action_node, int *dismiss_node, unsigned int *key) {
    const int prefix = cui__display_len(text);
    /* Keep a trailing hash from becoming part of the identity delimiter. A
       nonempty literal starting with hashes also needs a native body node. */
    const bool separate = prefix > 0 ? text[prefix - 1] == '#' : *text != '\0';
    const char *caption = cui__frame_printf(ui, "%.*s%s%s", prefix, text, separate ? " " : "", id);
    /* CUI measures the action string when deciding whether to stack it. */
    const char *button = action ? cui__frame_strdup(ui, *action && !cui__display_len(action) ? "x" : action, -1) : NULL;
    *key = cui_make_id(ui, caption) ^ 0xBA22u;
    cui_push_id(ui, caption);
    const cui_id action_key = button ? cui_make_id(ui, button) : 0;
    const cui_id dismiss_key = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Dismiss"));
    cui_pop_id(ui);
    const bool allowed = enabled && shown && !cui_is_disabled(ui);
    const fpas_owned_state *previous = cui__state_lazy(ui, fpas_owner_key(*key), sizeof *previous, false);
    const bool retain_focus = previous && previous->first && previous->first != action_key && ui->focus == previous->first;
    if (allowed && action_key && (focus == 1 || (focus == 0 && retain_focus)))
        ui->focus_next_request = action_key;
    if (allowed && focus == 2 && dismissible) ui->focus_next_request = dismiss_key;
    const cui_banner_opts options = {
        .tone = (cui_tone)tone, .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL,
        .action = button, .dismissible = dismissible != 0
    };
    bool open = shown != 0;
    const int start = ui->node_count;
    cui_push_disabled(ui, !enabled || !shown);
    const bool activated = cui_banner_ex(ui, caption, &open, &options);
    cui_pop_disabled(ui);
    *node = ui->node_count > start ? start : -1;
    *action_node = *node >= 0 && action ? fpas_find(ui, action_key) : -1;
    *dismiss_node = *node >= 0 && dismissible ? fpas_find(ui, dismiss_key) : -1;
    if (*node >= 0 && (separate || text[prefix])) fpas_text(ui, fpas_label_node(ui, *node), text);
    if (*action_node >= 0) {
        if (strstr(action, "##")) fpas_text(ui, fpas_label_node(ui, *action_node), action);
        fpas_banner_action_focus(ui, *action_node, action_key);
    }
    ui->last_node = *node;
    ui->last_id = *key;
    fpas_own(ui, *key, action_key, dismissible ? dismiss_key : 0, 0);
    return allowed ? (activated ? 1 : 0) | (!open ? 2 : 0) : 0;
}
