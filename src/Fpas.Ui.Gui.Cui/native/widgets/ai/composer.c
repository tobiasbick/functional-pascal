// [Documentation](../../../../../docs/ui/gui/voice/composer.md)
#include "../../fpas_cui.h"

static bool fpas_composer_has_text(const char *draft) {
    for (; *draft; ++draft)
        if (*draft != ' ' && *draft != '\n' && *draft != '\t') return true;
    return false;
}

static void fpas_composer_forget(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, (key ^ 0xF1E1Du) ^ 0xED17u);
}

/* CUI-133 (docs/future/cui-upstream-bugs.md#cui-133): the editor measures
   wrapped lines using its previous width. */
typedef struct {
    int text;
    float width;
} fpas_composer_layout;

static void fpas_composer_reflow(cui_ctx *ui, cui_node *node, void *data) {
    (void)node;
    const fpas_composer_layout *previous = data;
    if (fabsf(ui->nodes[previous->text].rect.w - previous->width) > 0.01f)
        cui_request_redraw(ui);
}

int fpas_cui_composer(cui_ctx *ui, const char *id, char *draft, size_t capacity, const char *placeholder,
    const char *const *attachments, int attachment_count, const cui_model *models, int model_count, int *selected,
    const float *levels, int level_count, float seconds, int max_lines, int width_kind, float width,
    int flags, int focus, int *removed, int *nodes) {
    if (!draft || capacity == 0) return 0;
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), field = key ^ 0xF1E1Du, send = key ^ 0x5E7Du;
    const cui_id mic = key ^ 0x41C0u, cancel = key ^ 0xCA9Cu, slot = key ^ 0x54465854u;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool busy = (flags & 2) != 0, microphone = (flags & 4) != 0, recording = (flags & 8) != 0;
    const int count = 6 + attachment_count;
    cui_id *parts = cui__frame_alloc(ui, sizeof *parts * (size_t)count);
    parts[0] = recording ? 0 : field;
    parts[1] = send;
    parts[2] = !recording && microphone ? mic : 0;
    parts[3] = recording ? cancel : 0;
    cui_push_id(ui, id);
    const cui_id model = cui_make_id(ui, "model");
    parts[4] = recording ? 0 : cui__hash_str(CUI_ICON_PLUS, cui_make_id(ui, "Attach files"));
    parts[5] = !recording && model_count > 0 ? model : 0;
    for (int i = 0; i < attachment_count; ++i) {
        cui_push_id_int(ui, i);
        parts[6 + i] = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Remove"));
        cui_pop_id(ui);
    }
    cui_pop_id(ui);
    const cui_id popup = model ^ 0x30DEu;
    cui_popup *menu = cui__popup_get(ui, popup, false);
    const bool show_models = enabled && !recording && model_count > 0;
    if (!show_models && menu && menu->open) cui__popup_set(ui, popup, false);
    const bool menu_top = menu && menu->open && cui__popup_is_topmost(ui, menu);
    /* CUI-043 (docs/future/cui-upstream-bugs.md#cui-043): an exiting model
       menu has no selectable highlight. */
    if (menu && (!menu_top || cui_key_pressed(ui, CUI_KEY_ESCAPE)) && cui_key_pressed(ui, CUI_KEY_ENTER)) {
        cui__menu_nav *nav = cui__state_lazy(ui, popup ^ 0x4E4156u, sizeof *nav, false);
        if (nav) nav->highlight = -1;
    }
    fpas_text_binding *binding = cui_state(ui, slot, sizeof *binding);
    cui__state_set_dtor(ui, slot, fpas_text_binding_dispose);
    fpas_own(ui, key, slot, field, popup);
    fpas_on_forget(ui, key, fpas_composer_forget);
    // CUI-110 (docs/future/cui-upstream-bugs.md#cui-110): keep the editor width until recording ends.
    const cui_id text_extent = field ^ 0xED17u;
    if (recording && cui__has_prev_rect(ui, text_extent)) (void)cui__state_entry(ui, text_extent);
    size_t length = strlen(draft);
    cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
    if (edit && binding->value && (capacity != binding->capacity || length != binding->length
        || memcmp(draft, binding->value, length) != 0)) {
        fpas_text_reset(ui, edit, (int)length);
    }
    if (!enabled) {
        // CUI-113 (docs/future/cui-upstream-bugs.md#cui-113): preserve Escape for the active control.
        fpas_clear_interaction(ui, field);
        fpas_clear_interaction(ui, send);
        fpas_clear_interaction(ui, cancel);
        fpas_clear_interaction(ui, mic);
    }
    const bool covered = enabled && cui__keys_covered(ui);
    if (enabled && !covered && focus >= 0 && focus < count && parts[focus])
        ui->focus_next_request = parts[focus];
    const bool enter = enabled && !recording && !covered && ui->focus == field && cui_key_pressed(ui, CUI_KEY_ENTER) &&
        !(ui->mods & (CUI_MOD_SHIFT | CUI_MOD_CTRL | CUI_MOD_SUPER));
    /* CUI-131 (docs/future/cui-upstream-bugs.md#cui-131): leave keyboard and
       text input with the popup above the composer. The open model menu
       uses its own keyboard navigation while its trigger keeps focus. */
    cui_id retained_focus = 0;
    if (covered && ui->focus && (ui->focus != model || !menu_top)) {
        const cui_id focused = ui->focus;
        bool owned = focused == field || focused == mic || focused == cancel;
        for (int i = 1; i < count; ++i) owned |= focused == parts[i];
        if (owned) {
            retained_focus = focused;
            const cui__cmp_state *state = cui__state_lazy(ui, key, sizeof *state, false);
            if (recording && (!state || !state->was_recording) && (focused == field || focused == mic))
                retained_focus = send;
            if (!recording && state && state->was_recording && (focused == cancel || focused == send))
                retained_focus = microphone ? mic : field;
            ui->focus = 0;
        }
    }
    const bool text_send = enabled && !recording && !busy;
    const bool had_text = text_send && fpas_composer_has_text(draft);
    if (enter) ui->kpressed[CUI_KEY_ENTER] = 0;
    const cui_composer_opts opts = { .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .busy = busy, .attachments = attachments, .attachment_count = attachment_count, .removed = removed,
        .models = models, .model_count = model_count, .model = selected, .max_lines = max_lines,
        .width = fpas_extent(width_kind, width), .mic = microphone, .recording = recording,
        .levels = levels, .level_count = level_count, .record_seconds = seconds };
    cui_push_disabled(ui, !enabled);
    int result = cui_chat_composer_ex(ui, id, draft, capacity, &opts);
    cui_pop_disabled(ui);
    if (retained_focus && !ui->focus) ui->focus = retained_focus;
    /* CUI-109 (docs/future/cui-upstream-bugs.md#cui-109): the keyboard and
       button submit the edited draft after queued input. */
    if (text_send) {
        const bool has_text = fpas_composer_has_text(draft);
        const bool clicked_send = ui->active == send && ui->hot == send && ui->mreleased[CUI_MOUSE_LEFT];
        if (!has_text) result &= ~CUI_COMPOSER_SEND;
        else if (enter || clicked_send) result |= CUI_COMPOSER_SEND;
        if (has_text != had_text) cui_request_redraw(ui);
    }
    /* CUI-173 (docs/future/cui-upstream-bugs.md#cui-173): cancel wins
       when Escape and Send activate the same recorder frame. */
    if (result & CUI_COMPOSER_CANCEL) result &= ~CUI_COMPOSER_VOICE;
    if (width_kind == 0) {
        ui->nodes[root].w = CUI_FIT;
        ui->nodes[root].self_align = CUI__ALIGN_NOSTRETCH;
    }
    const int text_node = recording ? -1 : fpas_find(ui, text_extent);
    if (text_node >= root) {
        fpas_composer_layout *previous = CUI_PAINT_DATA(ui, fpas_composer_layout);
        *previous = (fpas_composer_layout){ text_node, cui__prev_rect(ui, text_extent).w };
        ui->nodes[root].paint = fpas_composer_reflow;
        ui->nodes[root].paint_data = previous;
    }
    fpas_text_store(ui, binding, draft, strlen(draft), capacity);
    for (int i = 0; i < count; ++i) nodes[i] = parts[i] ? fpas_find(ui, parts[i]) : -1;
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
