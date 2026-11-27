// [Documentation](../../../../../docs/ui/gui/cui.md#data-and-marshalling)
static fpas_dev_result fpas_dev_empty(void) {
    return (fpas_dev_result){ .index = -1, .action = -1, .hover = -1 };
}

static void fpas_dev_reflow(cui_ctx *ui, cui_node *node, void *data) {
    const float previous = *(const float *)data;
    if (fabsf(node->rect.w - previous) > 0.01f) cui_request_redraw(ui);
}

/** CUI-133 (docs/future/cui-upstream-bugs.md#cui-133): reconcile width-dependent native layouts. */
static void fpas_dev_watch_width(cui_ctx *ui, int node, cui_id id) {
    float *width = cui__frame_alloc(ui, sizeof *width);
    *width = cui__prev_rect(ui, id).w;
    ui->nodes[node].paint = fpas_dev_reflow;
    ui->nodes[node].paint_data = width;
}

static cui_id fpas_dev_focused_item(cui_ctx *ui, cui_id group, int count) {
    if (ui->focus) for (int i = 0; i < count; ++i)
        if (ui->focus == cui__hash(&i, sizeof i, group)) return ui->focus;
    return 0;
}

/** Saves pending keys while a covered native control builds. */
static bool fpas_dev_save_keys(cui_ctx *ui, cui_id focus, uint8_t *saved) {
    /* CUI-131 (docs/future/cui-upstream-bugs.md#cui-131): these controls use focus without popup ownership. */
    if (!focus || (ui->focus != focus && ui->focus_next_request != focus) || !cui__keys_covered(ui)) return false;
    memcpy(saved, ui->kpressed, sizeof ui->kpressed);
    memset(ui->kpressed, 0, sizeof ui->kpressed);
    return true;
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
