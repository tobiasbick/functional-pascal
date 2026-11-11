/* Shared binding for CUI's single-line, search, and multiline editors. */
typedef int (*fpas_editor_draw)(cui_ctx *, const char *, char *, size_t, const cui_input_opts *);

typedef struct {
    char *value;
    size_t length;
    size_t capacity;
    bool password;
} fpas_text_binding;

static void fpas_text_binding_dispose(cui_ctx *ui, void *data) {
    fpas_text_binding *binding = data;
    if (!binding->value) return;
    volatile char *value = binding->value;
    for (size_t i = 0; i < binding->length; ++i) value[i] = 0;
    cui__free(ui, binding->value);
}

static void fpas_text_reset(cui_ctx *ui, cui_edit_state *edit, int length) {
    cui__edit_dtor(ui, edit);
    edit->undo_pos = 0;
    edit->last_edit = 0;
    edit->last_cursor = length;
    edit->last_kind = 0;
    edit->cursor = edit->anchor = length;
    edit->scroll_x = edit->scroll_y = 0;
    edit->preferred_x = -1;
    edit->selecting = false;
    edit->show_password = false;
}

static int fpas_draw_text_editor(cui_ctx *ui, const char *id, char *value, size_t capacity,
    cui_input_opts options, bool multiline, int enabled, int focus, int *nodes, int *revealed,
    fpas_editor_draw draw) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id binding_key = key ^ 0x54465854u;
    const cui_id area_key = key ^ 0xED17u;
    fpas_text_binding *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__state_set_dtor(ui, binding_key, fpas_text_binding_dispose);
    fpas_own(ui, key, binding_key, multiline ? area_key : 0, 0);

    size_t length = strlen(value);
    cui_edit_state *edit = cui__state_lazy(ui, key, sizeof *edit, false);
    if (edit && binding->value && (capacity != binding->capacity || options.password != binding->password
        || length != binding->length || memcmp(value, binding->value, length) != 0)) {
        /* An external replacement invalidates native byte offsets and undo history. */
        fpas_text_reset(ui, edit, (int)length);
    }
    const bool was_revealed = edit && edit->show_password;
    cui_push_id_int(ui, (int)key);
    const cui_id clear = cui_make_id(ui, CUI_ICON_X);
    const cui_id show = cui__hash_str(CUI_ICON_EYE, cui_make_id(ui, "Show password"));
    const cui_id hide = cui__hash_str(CUI_ICON_EYE_OFF, cui_make_id(ui, "Hide password"));
    cui_pop_id(ui);
    const cui_id eye = was_revealed ? hide : show;
    if (ui->focus == show || ui->focus == hide) ui->focus = options.password ? eye : key;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    if (options.placeholder)
        options.placeholder = cui__frame_strdup(ui, options.placeholder, (int)strlen(options.placeholder));

    /* Disable all editor affordances within the same scope. */
    cui_push_disabled(ui, !enabled);
    const int flags = draw(ui, id, value, capacity, &options);
    cui_pop_disabled(ui);

    nodes[0] = fpas_find(ui, key);
    nodes[1] = options.clearable ? fpas_find(ui, clear) : -1;
    nodes[2] = options.password ? fpas_find(ui, eye) : -1;
    const int root = options.label || options.hint || options.error ? ui->nodes[nodes[0]].parent : nodes[0];
    if (options.label && strstr(options.label, "##")) fpas_text(ui, ui->nodes[root].first, options.label);
    ui->last_node = root;
    ui->last_id = key;
    edit = cui__state_lazy(ui, key, sizeof *edit, false);
    /* Escape can release focus after CUI marks this editor active for the frame. */
    if (ui->edit == edit && ui->focus != key) ui->edit = NULL;
    *revealed = options.password && edit->show_password;

    length = strlen(value);
    /* CUI builds the text paint data before processing the clear and reveal buttons. */
    const int area = fpas_find(ui, area_key);
    cui__edit_paint *paint = ui->nodes[area].paint_data;
    if ((size_t)paint->len != length) {
        paint->buf = cui__frame_strdup(ui, value, (int)length);
        paint->len = (int)length;
        paint->anchor = edit->anchor;
        paint->cursor = edit->cursor;
        paint->scroll_x = edit->scroll_x;
        if (multiline) {
            const float width = cui__prev_rect(ui, area_key).w;
            paint->line_count = cui__edit_lines(ui, paint->font, paint->size, paint->buf,
                paint->len, width > 0 ? width : 200, &paint->lines);
            paint->scroll_y = edit->scroll_y = 0;
        }
    }
    if (options.password && was_revealed != edit->show_password) {
        paint->password = !edit->show_password;
        cui__wakeup_after(ui, 0.001f);
    }
    if (multiline && allowed) ui->nodes[area].paint = fpas_paint_text_area;

    if (!binding->value || length != binding->length || memcmp(value, binding->value, length) != 0) {
        fpas_text_binding_dispose(ui, binding);
        binding->value = cui__alloc(ui, length + 1);
        memcpy(binding->value, value, length + 1);
        binding->length = length;
    }
    binding->capacity = capacity;
    binding->password = options.password;
    return allowed ? flags & (CUI_CHANGED | CUI_SUBMITTED) : 0;
}
