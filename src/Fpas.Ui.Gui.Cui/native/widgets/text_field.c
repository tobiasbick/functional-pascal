#include "../fpas_cui.h"

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
    edit->scroll_x = 0;
    edit->selecting = false;
    edit->show_password = false;
}

int fpas_cui_text_field(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, int password, int clearable,
    int read_only, int enabled, int focus, int *nodes, int *revealed) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id binding_key = key ^ 0x54465854u;
    fpas_text_binding *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__state_set_dtor(ui, binding_key, fpas_text_binding_dispose);
    fpas_own(ui, key, binding_key, 0, 0);

    size_t length = strlen(value);
    cui_edit_state *edit = cui__state_lazy(ui, key, sizeof *edit, false);
    if (edit && binding->value && (capacity != binding->capacity || (password != 0) != binding->password
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
    if (ui->focus == show || ui->focus == hide) ui->focus = password ? eye : key;
    if (enabled && focus) ui->focus_next_request = key;

    const cui_input_opts options = {
        .label = label, .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, (int)strlen(placeholder)) : NULL,
        .icon = icon, .hint = hint, .error = error, .prefix = prefix, .suffix = suffix,
        .size = (cui_control_size)size, .password = password != 0, .clearable = clearable != 0,
        .read_only = read_only != 0
    };
    /* Disable the eye as well as the editor and the clear button. */
    cui_push_disabled(ui, !enabled);
    const int flags = cui_input_ex(ui, id, value, capacity, &options);
    cui_pop_disabled(ui);

    nodes[0] = fpas_find(ui, key);
    nodes[1] = clearable ? fpas_find(ui, clear) : -1;
    nodes[2] = password ? fpas_find(ui, eye) : -1;
    const int root = label || hint || error ? ui->nodes[nodes[0]].parent : nodes[0];
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;
    edit = cui__state_lazy(ui, key, sizeof *edit, false);
    /* Escape can release focus after CUI marks this editor active for the frame. */
    if (ui->edit == edit && ui->focus != key) ui->edit = NULL;
    *revealed = password && edit->show_password;

    length = strlen(value);
    /* CUI builds the text paint data before processing the clear and reveal buttons. */
    const int area = fpas_find(ui, key ^ 0xED17u);
    cui__edit_paint *paint = ui->nodes[area].paint_data;
    if ((size_t)paint->len != length) {
        paint->buf = cui__frame_strdup(ui, value, (int)length);
        paint->len = (int)length;
        paint->anchor = edit->anchor;
        paint->cursor = edit->cursor;
        paint->scroll_x = edit->scroll_x;
    }
    if (password && was_revealed != edit->show_password) {
        paint->password = !edit->show_password;
        cui__wakeup_after(ui, 0.001f);
    }

    if (!binding->value || length != binding->length || memcmp(value, binding->value, length) != 0) {
        fpas_text_binding_dispose(ui, binding);
        binding->value = cui__alloc(ui, length + 1);
        memcpy(binding->value, value, length + 1);
        binding->length = length;
    }
    binding->capacity = capacity;
    binding->password = password != 0;
    return enabled ? flags & (CUI_CHANGED | CUI_SUBMITTED) : 0;
}
