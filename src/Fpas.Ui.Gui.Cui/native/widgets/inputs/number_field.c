#include "../../fpas_cui.h"

void fpas_cui_number_field(cui_ctx *ui, const char *id, double *value, double min, double max,
    const char *label, const char *suffix, double step, int decimals, int enabled, int focus, int *nodes) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id number_key = key ^ 0x4E554Du;
    cui__number_state *state = cui_state(ui, number_key, sizeof *state);
    fpas_own(ui, key, number_key, 0, 0);
    const double original = *value;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (state->init && state->last != original) {
        /* Reconcile an external MVU replacement, including while editing. */
        snprintf(state->text, sizeof state->text, "%.*f", decimals, original);
        cui_edit_state *edit = cui__state_lazy(ui, key, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, (int)strlen(state->text));
        state->last = original;
    }
    if (allowed && focus) ui->focus_next_request = key;
    const cui_number_opts options = { .label = label, .suffix = suffix, .step = step,
        .decimals = decimals, .disabled = !allowed, .width = CUI_FILL };
    cui_push_disabled(ui, !enabled);
    cui_number_ex(ui, id, value, min, max, &options);
    cui_pop_disabled(ui);

    const int field = fpas_find(ui, key);
    const int row = ui->nodes[field].parent;
    const int root = label ? ui->nodes[row].parent : row;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);

    nodes[0] = field;
    cui_push_id(ui, id);
    nodes[1] = fpas_find(ui, cui__hash_str(CUI_ICON_MINUS, cui_make_id(ui, "Decrease")));
    nodes[2] = fpas_find(ui, cui__hash_str(CUI_ICON_PLUS, cui_make_id(ui, "Increase")));
    cui_pop_id(ui);
    ui->last_node = root;
    ui->last_id = key;
}
