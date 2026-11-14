#include "../../fpas_cui.h"

int fpas_cui_seed_field(cui_ctx *ui, const char *id,
    const char *label, uint32_t *seed, int *locked, const char *display, const char *random, const char *fixed, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label);
    const cui_id dice = key ^ 0xD1CEu, lock = key ^ CUI__WELL_TOOL_SALT;
    bool current = locked && *locked != 0;
    if (enabled && focus == 0 && !current) ui->focus_next_request = dice;
    if (enabled && focus == 1 && locked) ui->focus_next_request = lock;
    const cui_seed_field_opts options = { .value = display, .random_text = random, .fixed_text = fixed,
        .plain = (flags & 8) != 0 };
    cui_push_disabled(ui, !enabled);
    const int result = cui_seed_field_ex(ui, label, seed, locked ? &current : NULL, &options);
    cui_pop_disabled(ui);
    if (locked) *locked = current;
    parts[0] = fpas_find(ui, dice);
    parts[1] = locked ? fpas_find(ui, lock) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
