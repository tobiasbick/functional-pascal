#include "../../fpas_cui.h"

void fpas_cui_readout(cui_ctx *ui, const char *id, const char *label, const char *value,
    const char *unit, const char *detail, int size, int tone, int accent, int align,
    int width_kind, float width) {
    const int node = ui->node_count;
    const cui_readout_opts options = { .unit = unit, .detail = detail, .size = size,
        .tone = tone, .accent = accent != 0, .align = align, .width = fpas_extent(width_kind, width) };
    cui_readout_ex(ui, label, value, &options);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = cui_make_id(ui, id);
}
