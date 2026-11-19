// [Documentation](../../../../../docs/ui/gui/motion/rolling-number.md)
#include "../../fpas_cui.h"

void fpas_cui_number_roll(cui_ctx *ui, const char *id, const char *text, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_text_fx_opts options = fpas_motion_options(o);
    cui_number_roll_ex(ui, id, text, &options);
    fpas_motion_node(ui, node, key);
}
