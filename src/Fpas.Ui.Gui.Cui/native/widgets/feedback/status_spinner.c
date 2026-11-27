// [Documentation](../../../../../docs/ui/gui/feedback/status-spinner.md)
#include "../../fpas_cui.h"

/** Draws native activity or result feedback, reporting its visibility. */
int fpas_cui_status_spinner(cui_ctx *ui, const char *id, int status, float size,
    int theme_token, uint32_t rgba, int keep) {
    const int first = ui->node_count;
    const cui_status_spinner_opts options = { .size = size,
        .color = fpas_spinner_color(theme_token, rgba), .keep = keep != 0 };
    /* Give the native result timer the normal CUI lifetime across brief view changes. */
    cui_push_id(ui, "#status-spinner-clock");
    const bool visible = cui_status_spinner_ex(ui, id, status, &options);
    cui_pop_id(ui);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
    return visible;
}
