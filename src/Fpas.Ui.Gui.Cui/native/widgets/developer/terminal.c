// [Documentation](../../../../../docs/ui/gui/files/terminal.md)
#include "../../fpas_cui.h"

typedef struct {
    cui_terminal_buffer buffer;
    char *source;
    size_t length;
    int revision;
} fpas_terminal_state;

typedef struct {
    cui__tm_paint *native;
    cui_rect previous;
    bool enabled;
} fpas_terminal_paint;

/** Releases the native scrollback and the last application output. */
static void fpas_terminal_free(cui_ctx *ui, void *data) {
    fpas_terminal_state *state = data;
    cui__free(ui, state->buffer.text);
    cui__free(ui, state->buffer.lines);
    cui__free(ui, state->source);
}

/** Releases the terminal's remaining child state on retirement. */
static void fpas_terminal_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, cui__hash_str("#term-input", key));
    fpas_forget_state(ui, key ^ 0x7E2Du);
    fpas_forget_state(ui, (key ^ 0x7E2Eu) ^ CUI__WHEEL_XFORM);
}

/** Paints with CUI and schedules a new wrap calculation after a geometry change. */
static void fpas_paint_terminal(cui_ctx *ui, cui_node *node, void *data) {
    const fpas_terminal_paint *paint = data;
    /* CUI-123 (docs/future/cui-upstream-bugs.md#cui-123): native wrapping uses the previous viewport. */
    if (node->rect.w != paint->previous.w || node->rect.h != paint->previous.h) cui_request_redraw(ui);
    const int hits = ui->hit_count;
    cui__paint_terminal(ui, node, paint->native);
    if (!paint->enabled) ui->hit_count = hits;
}

/** Replays model output into CUI's bounded scrollback and returns native input. */
const char *fpas_cui_terminal(cui_ctx *ui, const char *id, const char *output, int revision, const char *prompt,
    int width_kind, float width, float height, float size, int byte_capacity, int line_capacity, int flags, int *interrupt) {
    *interrupt = 0;
    if (byte_capacity < 64 || line_capacity < 2 || (size_t)line_capacity > SIZE_MAX / sizeof(cui_terminal_line))
        return NULL;
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0x7E2F01u;
    const cui_id view = key ^ 0x7E2Du, wheel = key ^ 0x7E2Eu;
    const cui_id focus = cui__hash_str("#term-input", key);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool input = (flags & 2) != 0;
    fpas_own(ui, key, slot, wheel, 0);
    fpas_on_forget(ui, key, fpas_terminal_retire);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = input ? focus : view;
    fpas_terminal_state *state = cui_state(ui, slot, sizeof *state);
    const bool configure = state->buffer.text_cap != (unsigned)byte_capacity || state->buffer.line_cap != line_capacity;
    if (configure) {
        cui__free(ui, state->buffer.text);
        cui__free(ui, state->buffer.lines);
        char *text = cui__alloc(ui, (size_t)byte_capacity);
        cui_terminal_line *lines = cui__alloc(ui, (size_t)line_capacity * sizeof *lines);
        cui_terminal_init(&state->buffer, text, (size_t)byte_capacity, lines, line_capacity);
        cui__state_set_dtor(ui, slot, fpas_terminal_free);
    }

    const size_t length = strlen(output);
    size_t prefix = state->length;
    const bool replace = configure || state->revision != revision || length < prefix ||
        (prefix && memcmp(state->source, output, prefix));
    cui__tm_state *terminal = cui__state_lazy(ui, key, sizeof *terminal, false);
    if (replace) {
        if (!configure) cui_terminal_clear(&state->buffer);
        prefix = 0;
        if (terminal) {
            terminal->top = 0;
            terminal->follow = true;
            terminal->has_sel = terminal->selecting = false;
        }
    }
    if (length > prefix) cui_terminal_write(&state->buffer, output + prefix, -1);
    if (length != state->length || length > prefix) {
        state->source = cui__realloc(ui, state->source, length + 1);
        memcpy(state->source, output, length + 1);
    }
    state->length = length;
    state->revision = revision;

    if (!enabled) {
        /* CUI-122 (docs/future/cui-upstream-bugs.md#cui-122): native shortcuts bypass the disabled scope. */
        fpas_clear_interaction(ui, focus);
        fpas_clear_interaction(ui, view);
        if (ui->scroll_target == wheel) ui->scroll_target = 0;
        if (terminal) terminal->selecting = false;
    } else if (!input && ui->focus == focus) {
        ui->focus = view;
    }
    if (enabled && input && (flags & 4)) ui->focus_next_request = focus;
    const cui_terminal_opts options = { .prompt = prompt, .width = fpas_extent(width_kind, width),
        .height = height, .size = size, .no_input = !input };
    cui_push_disabled(ui, !enabled);
    const cui_terminal_result result = cui_terminal_ex(ui, id, &state->buffer, &options);
    cui_pop_disabled(ui);
    cui_node *output_node = &ui->nodes[ui->nodes[root].first];
    fpas_terminal_paint *paint = cui__frame_alloc(ui, sizeof *paint);
    *paint = (fpas_terminal_paint){ output_node->paint_data, cui__prev_rect(ui, view), enabled };
    output_node->paint = fpas_paint_terminal;
    output_node->paint_data = paint;
    if (!enabled) output_node->flags &= ~CUI_NF_INTERACTIVE;
    fpas_creative_node(ui, root, key);
    *interrupt = enabled && result.interrupt;
    return enabled ? result.command : NULL;
}
