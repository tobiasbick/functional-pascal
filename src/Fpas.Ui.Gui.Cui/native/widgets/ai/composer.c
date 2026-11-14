#include "../../fpas_cui.h"

int fpas_cui_composer(cui_ctx *ui, const char *id, char *draft, size_t capacity, const char *placeholder,
    const char *const *attachments, int attachment_count, const cui_model *models, int model_count, int *selected,
    const float *levels, int level_count, float seconds, int max_lines, int width_kind, float width,
    int flags, int focus, int *removed, int *nodes) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), field = key ^ 0xF1E1Du, send = key ^ 0x5E7Du;
    const cui_id mic = key ^ 0x41C0u, cancel = key ^ 0xCA9Cu, slot = key ^ 0x54465854u;
    const cui_id parts[] = { field, send, mic, cancel };
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool recording = (flags & 8) != 0;
    cui_push_id(ui, id);
    const cui_id model = cui_make_id(ui, "model");
    cui_pop_id(ui);
    const cui_id popup = model ^ 0x30DEu;
    if (!enabled || recording || model_count == 0) cui__popup_set(ui, popup, false);
    fpas_text_binding *binding = cui_state(ui, slot, sizeof *binding);
    cui__state_set_dtor(ui, slot, fpas_text_binding_dispose);
    fpas_own(ui, key, slot, field, popup);
    size_t length = strlen(draft);
    cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
    if (edit && binding->value && (capacity != binding->capacity || length != binding->length
        || memcmp(draft, binding->value, length) != 0)) {
        fpas_text_reset(ui, edit, (int)length);
    }
    if (!enabled && ui->focus == field) ui->focus = 0;
    if (enabled && focus >= 0 && focus < 4) ui->focus_next_request = parts[focus];
    const cui_composer_opts opts = { .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .busy = (flags & 2) != 0, .attachments = attachments, .attachment_count = attachment_count, .removed = removed,
        .models = models, .model_count = model_count, .model = selected, .max_lines = max_lines,
        .width = fpas_extent(width_kind, width), .mic = (flags & 4) != 0, .recording = recording,
        .levels = levels, .level_count = level_count, .record_seconds = seconds };
    cui_push_disabled(ui, !enabled);
    const int result = cui_chat_composer_ex(ui, id, draft, capacity, &opts);
    cui_pop_disabled(ui);
    length = strlen(draft);
    if (binding->length != length || !binding->value || memcmp(binding->value, draft, length) != 0) {
        fpas_text_binding_dispose(ui, binding);
        binding->value = cui__alloc(ui, length + 1);
        memcpy(binding->value, draft, length + 1);
        binding->length = length;
    }
    binding->capacity = capacity;
    nodes[0] = fpas_find(ui, field);
    nodes[1] = fpas_find(ui, send);
    nodes[2] = fpas_find(ui, mic);
    nodes[3] = fpas_find(ui, cancel);
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
