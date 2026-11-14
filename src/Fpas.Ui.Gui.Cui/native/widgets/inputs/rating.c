#include "../../fpas_cui.h"

int fpas_cui_rating(cui_ctx *ui, const char *id, int value, int maximum, int enabled) {
    const bool allowed = enabled && !cui_is_disabled(ui);
    int proposed = value;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_rating(ui, id, &proposed, maximum);
    cui_pop_disabled(ui);
    const cui_id key = cui_make_id(ui, id);
    const int row = fpas_find(ui, key);
    if (!allowed) ui->nodes[row].opacity = CUI__DISABLED;
    ui->last_node = row;
    ui->last_id = key;
    return changed ? proposed : -1;
}
