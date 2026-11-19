// [Documentation](../../../../../docs/ui/gui/smart-inputs/scrub-field.md)
#include "../../fpas_cui.h"

typedef struct { float proposed; bool init; } fpas_scrub_binding;

void fpas_cui_scrub_field(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, const char *format, const char *suffix, const char *icon,
    int width_kind, float width, int enabled, int focus, int *node, unsigned int *key) {
    *key = cui_make_id(ui, id);
    const cui_id field = *key ^ 0x5C2Bu, binding_key = *key ^ 0x53435242u;
    fpas_scrub_binding *binding = cui_state(ui, binding_key, sizeof *binding);
    cui__scrub_state *state = cui_state(ui, *key, sizeof *state);
    fpas_own(ui, *key, binding_key, field, 0);
    if (binding->init && binding->proposed != *value) {
        state->editing = false; state->start = *value; state->press_x = ui->mx;
        cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, 0);
        if (ui->focus == field) ui->focus = *key;
    }
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed) {
        state->editing = false;
        if (ui->focus == *key || ui->focus == field) ui->focus = 0;
    } else if (focus) ui->focus_next_request = state->editing ? field : *key;
    const bool typing = state->editing;
    const cui_scrub_opts options = { .step = step, .format = format,
        .suffix = suffix ? cui__frame_strdup(ui, suffix, -1) : NULL,
        .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL,
        .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !allowed);
    cui_scrub_ex(ui, id, value, minimum, maximum, &options);
    cui_pop_disabled(ui);
    *node = fpas_find(ui, typing ? field : *key);
    if (!icon && !typing) {
        const int caption = fpas_label_node(ui, *node);
        if (caption >= 0) fpas_text(ui, caption, label);
    } else if (!icon) {
        /* The typed mode uses a literal prefix rather than the hashed identity. */
        const int prefix = fpas_label_node(ui, *node);
        if (prefix >= 0) fpas_text(ui, prefix, label);
    }
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(*key), sizeof *owner);
    owner->focus = field;
    binding->proposed = *value; binding->init = true;
    ui->last_node = *node; ui->last_id = *key;
}
