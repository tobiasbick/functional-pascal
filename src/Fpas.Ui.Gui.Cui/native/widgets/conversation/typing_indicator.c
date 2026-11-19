// [Documentation](../../../../../docs/ui/gui/social-and-media/typing-indicator.md)
#include "../../fpas_cui.h"

void fpas_cui_typing_indicator(cui_ctx *ui, const char *id, const char *name) {
    const int first = ui->node_count;
    cui_typing_indicator(ui, name);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
