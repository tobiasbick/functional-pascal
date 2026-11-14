#include "../../fpas_cui.h"

typedef struct { char *draft, *model; size_t capacity; } fpas_inline_binding;

static void fpas_inline_dispose(cui_ctx *ui, void *data) {
    fpas_inline_binding *binding = data;
    cui__free(ui, binding->draft); cui__free(ui, binding->model);
}

int fpas_cui_inline_edit(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *placeholder, int font, float size, int width_kind, float width,
    int enabled, int focus, int *editing, int *node, unsigned int *key) {
    *key = cui_make_id(ui, id);
    const cui_id field = *key ^ 0x1D17u, binding_key = *key ^ 0x494E4C4Eu;
    fpas_inline_binding *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__state_set_dtor(ui, binding_key, fpas_inline_dispose);
    fpas_own(ui, *key, binding_key, field, 0);
    cui__inline_state *state = cui_state(ui, *key, sizeof *state);
    bool replace = !binding->draft || binding->capacity != capacity || strcmp(binding->model, value);
    if (replace) {
        if (binding->capacity != capacity) {
            fpas_inline_dispose(ui, binding);
            binding->draft = cui__alloc(ui, capacity); binding->model = cui__alloc(ui, capacity);
            binding->capacity = capacity;
        }
        snprintf(binding->draft, capacity, "%s", value);
        snprintf(binding->model, capacity, "%s", value);
        snprintf(state->backup, sizeof state->backup, "%s", value);
        cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, (int)strlen(value));
    }
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed) {
        state->editing = false;
        snprintf(binding->draft, capacity, "%s", value);
        if (fpas_has_focus(ui, *key) || ui->focus == field) ui->focus = 0;
    } else if (focus) ui->focus_next_request = state->editing ? field : *key;
    const cui_inline_edit_opts options = { .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .font = font, .size = size, .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !allowed);
    const int result = cui_inline_edit_ex(ui, id, binding->draft, capacity, &options);
    cui_pop_disabled(ui);
    *node = fpas_find(ui, *key);
    *editing = state->editing;
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(*key), sizeof *owner);
    owner->focus = field;
    if (result & CUI_CHANGED) snprintf(binding->model, capacity, "%s", binding->draft);
    snprintf(value, capacity, "%s", binding->draft);
    ui->last_node = *node; ui->last_id = *key;
    return allowed ? result : 0;
}
