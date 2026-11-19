// [Documentation](../../../../../docs/ui/gui/cui.md#data-and-marshalling)
static fpas_dev_result fpas_dev_empty(void) {
    return (fpas_dev_result){ .index = -1, .action = -1, .hover = -1 };
}

static cui_id fpas_dev_text_slot(cui_id field) {
    return field ^ 0x54465854u;
}

static fpas_text_binding *fpas_dev_bind_text(cui_ctx *ui, cui_id field, const char *text,
    size_t capacity, bool code) {
    const cui_id slot = fpas_dev_text_slot(field);
    fpas_text_binding *binding = cui_state(ui, slot, sizeof *binding);
    cui__state_set_dtor(ui, slot, fpas_text_binding_dispose);
    if (!binding->value || (binding->capacity == capacity && strcmp(binding->value, text) == 0))
        return binding;

    const int length = (int)strlen(text);
    if (code) {
        cui__ed_state *edit = cui__state_lazy(ui, field ^ CUI__ED_SALT, sizeof *edit, false);
        if (edit) {
            fpas_text_reset(ui, &edit->ed, length);
            edit->pref_col = -1;
        }
    } else {
        cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, length);
    }
    return binding;
}
