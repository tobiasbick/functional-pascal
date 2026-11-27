// [Documentation](../../../../../docs/ui/gui/instruments/readout.md)
#include "../../fpas_cui.h"

/** Draws the native readout with logical and physical text alignment. */
void fpas_cui_readout(cui_ctx *ui, const char *id, const char *label, const char *value,
    const char *unit, const char *detail, int size, int tone, int accent, int align,
    int width_kind, float width) {
    const int node = ui->node_count;
    const cui_readout_opts options = { .unit = unit, .detail = detail, .size = size,
        .tone = tone, .accent = accent != 0, .align = align, .width = fpas_extent(width_kind, width) };
    cui_readout_ex(ui, label, value, &options);
    /* CUI-153 (docs/future/cui-upstream-bugs.md#cui-153): Resolve physical placement
       after the layout determines the reading direction for the text and value row. */
    if (align == CUI_TEXT_LEFT || align == CUI_TEXT_RIGHT) {
        ui->nodes[node].align = CUI_ALIGN_STRETCH;
        for (int child = ui->nodes[node].first; child >= 0; child = ui->nodes[child].next)
            ui->nodes[child].text_align = (cui_text_align)align;
    }
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = cui_make_id(ui, id);
}
