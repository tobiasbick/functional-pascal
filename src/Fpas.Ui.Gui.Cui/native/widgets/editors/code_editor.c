#include "../../fpas_cui.h"

typedef struct {
    fpas_text_binding text;
    int line, column, lines;
    bool refused;
} fpas_code_editor_state;

static void fpas_code_editor_dispose(cui_ctx *ui, void *data) {
    fpas_code_editor_state *state = data;
    fpas_text_binding_dispose(ui, &state->text);
}

static void fpas_code_editor_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, key ^ 0xED15C011u);
}

int fpas_cui_code_editor(cui_ctx *ui, const char *id, char *text, size_t capacity, const char *language, const char *placeholder, int width_kind, float width, float height, float size, int tab_size, int flags, int *line, int *column, int *lines) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0xC0DE07u, editor = key ^ CUI__ED_SALT;
    const cui_id focus = key ^ 0xED1B0D1u;
    fpas_own(ui, key, slot, editor, 0);
    fpas_on_forget(ui, key, fpas_code_editor_retire);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = focus;
    fpas_code_editor_state *state = cui_state(ui, slot, sizeof *state);
    cui__state_set_dtor(ui, slot, fpas_code_editor_dispose);

    fpas_text_binding *prior = &state->text;
    if (prior->value && (prior->capacity != capacity || strcmp(prior->value, text))) {
        cui__ed_state *edit = cui__state_lazy(ui, editor, sizeof *edit, false);
        if (edit) {
            fpas_text_reset(ui, &edit->ed, (int)strlen(text));
            edit->pref_col = -1;
        }
    }
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 16)) ui->focus_next_request = focus;
    const cui_code_editor_opts options = { .lang = language,
        .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .width = fpas_extent(width_kind, width), .height = height, .size = size, .tab_size = tab_size,
        .read_only = !enabled || (flags & 2), .use_tabs = (flags & 4) != 0, .no_line_numbers = (flags & 8) != 0 };
    cui_push_disabled(ui, !enabled);
    const cui_code_editor_result event = cui_code_editor_ex(ui, id, text, capacity, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    if (!prior->value || strcmp(prior->value, text)) {
        fpas_text_binding_dispose(ui, prior);
        prior->length = strlen(text);
        prior->value = cui__alloc(ui, prior->length + 1);
        memcpy(prior->value, text, prior->length + 1);
    }
    prior->capacity = capacity;
    const bool moved = state->line != event.line || state->column != event.column || state->lines != event.line_count;
    const bool refusal_changed = state->refused != event.refused;
    state->refused = event.refused;
    *line = state->line = event.line;
    *column = state->column = event.column;
    *lines = state->lines = event.line_count;
    return (enabled && event.changed ? 1 : 0) | (enabled && event.refused ? 2 : 0) | (moved || refusal_changed ? 4 : 0);
}
