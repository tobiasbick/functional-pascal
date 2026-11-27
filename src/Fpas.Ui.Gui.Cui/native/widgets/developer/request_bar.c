// [Documentation](../../../../../docs/ui/gui/more-developer-tools/http-request-bar.md)
#include "../../fpas_cui.h"

/** Return whether focus belongs to any native presentation of the action. */
static bool fpas_request_action_has_focus(cui_id key, cui_id focus) {
    return focus && (focus == cui__hash_str("Send", key) || focus == cui__hash_str("Cancel", key) ||
        focus == cui__hash_str(CUI_ICON_SEND, key) || focus == cui__hash_str(CUI_ICON_X, key));
}

/** Retire the method popup and URL edit history with the request bar. */
static void fpas_request_bar_retire(cui_ctx *ui, cui_id key) {
    const cui_id method = cui__hash_str("method", key);
    const cui_id field = cui__hash_str("url", key);
    fpas_forget_state(ui, method);
    fpas_forget_state(ui, (method ^ 0x5E1Eu) ^ 0x4E4156u);
    fpas_forget_state(ui, field);
}

/** Draw the native request bar and return its edit and action flags. */
void fpas_cui_request_bar(cui_ctx *ui, const char *id, int method, char *url, size_t capacity, int loading, const char *placeholder, int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id field = cui__hash_str("url", key);
    const cui_id trigger = cui__hash_str("method", key);
    const cui_id popup = trigger ^ 0x5E1Eu;
    const bool allowed = (flags & 1) && !cui_is_disabled(ui) && url && capacity;
    const float previous_width = cui__prev_rect(ui, key).w;
    const bool narrow = previous_width > 0 && previous_width < 420;
    const cui_id action = cui__hash_str(narrow ? (loading ? CUI_ICON_X : CUI_ICON_SEND) :
        (loading ? "Cancel" : "Send"), key);
    *out = fpas_dev_empty();
    fpas_own(ui, key, fpas_dev_text_slot(field), 0, popup);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    fpas_on_forget(ui, key, fpas_request_bar_retire);
    if (!allowed) cui__popup_set(ui, popup, false);
    if (allowed && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = field;
    /* CUI-149 (docs/future/cui-upstream-bugs.md#cui-149): the action's ID follows its label or icon. */
    if (fpas_request_action_has_focus(key, ui->focus))
        ui->focus = allowed && (loading || url[0]) ? action : 0;
    fpas_text_binding *binding = url && capacity ? fpas_dev_bind_text(ui, field, url, capacity, false) : NULL;
    cui_popup *menu = cui__popup_get(ui, popup, false);
    cui_id focused = ui->focus == action || ui->focus == trigger ? ui->focus : field;
    if (focused == trigger && menu && menu->open && cui__popup_is_topmost(ui, menu)) focused = 0;
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = allowed && fpas_dev_save_keys(ui, focused, keys);
    const int text_length = ui->text_in_len;
    if (muted) ui->text_in_len = 0;
    cui_push_disabled(ui, !allowed);
    const cui_request_bar_opts o = { .width = fpas_extent(width_kind, width), .loading = loading,
        .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL };
    const cui_request_bar_result r = cui_request_bar_ex(ui, id, &method, url, capacity, &o);
    out->index = method;
    if (allowed) out->flags = (r.changed ? 1 : 0) | (r.send ? 2 : 0) | (r.cancel ? 4 : 0);
    cui_pop_disabled(ui);
    if (muted) {
        memcpy(ui->kpressed, keys, sizeof keys);
        ui->text_in_len = text_length;
    }
    if (binding) fpas_text_store(ui, binding, url, strlen(url), capacity);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    owner->focus = ui->focus == action || ui->focus == trigger ? ui->focus : field;
    ui->nodes[root].w = o.width;
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui,root,key);
}
