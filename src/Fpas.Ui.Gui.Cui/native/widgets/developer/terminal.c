#include "../../fpas_cui.h"

typedef struct {
    cui_terminal_buffer buffer;
    char *source;
    size_t length;
    int revision;
} fpas_terminal_state;

static void fpas_terminal_free(cui_ctx *ui, void *data) {
    fpas_terminal_state *state = data;
    cui__free(ui, state->buffer.text);
    cui__free(ui, state->buffer.lines);
    cui__free(ui, state->source);
}

static void fpas_terminal_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, cui__hash_str("#term-input", key));
}

const char *fpas_cui_terminal(cui_ctx *ui, const char *id, const char *output, int revision, const char *prompt, int width_kind, float width, float height, float size, int flags, int *interrupt) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0x7E2F01u;
    const cui_id focus = cui__hash_str("#term-input", key);
    fpas_own(ui, key, slot, 0, 0);
    fpas_on_forget(ui, key, fpas_terminal_retire);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = focus;
    fpas_terminal_state *state = cui_state(ui, slot, sizeof *state);
    if (!state->buffer.line_cap) {
        char *text = cui__alloc(ui, 64 * 1024);
        cui_terminal_line *lines = cui__alloc(ui, 1024 * sizeof *lines);
        cui_terminal_init(&state->buffer, text, 64 * 1024, lines, 1024);
        cui__state_set_dtor(ui, slot, fpas_terminal_free);
        state->revision = revision;
    }

    const size_t length = strlen(output);
    size_t prefix = state->length;
    if (state->revision != revision || length < prefix || (prefix && memcmp(state->source, output, prefix))) {
        cui_terminal_clear(&state->buffer);
        prefix = 0;
    }
    if (length > prefix) cui_terminal_write(&state->buffer, output + prefix, -1);
    if (length != state->length || length > prefix) {
        state->source = cui__realloc(ui, state->source, length + 1);
        memcpy(state->source, output, length + 1);
    }
    state->length = length;
    state->revision = revision;

    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool input = (flags & 2) != 0;
    if (enabled && input && (flags & 4)) ui->focus_next_request = focus;
    const cui_terminal_opts options = { .prompt = prompt, .width = fpas_extent(width_kind, width),
        .height = height, .size = size, .no_input = !input };
    cui_push_disabled(ui, !enabled);
    const cui_terminal_result result = cui_terminal_ex(ui, id, &state->buffer, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *interrupt = enabled && result.interrupt;
    return enabled ? result.command : NULL;
}
