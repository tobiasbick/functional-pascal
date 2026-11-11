#include "../fpas_cui.h"

static void fpas_tooltip_init(cui_ctx *ui) {
    /* The first tooltip has no earlier tooltip from which to inherit a warm delay. */
    cui__tooltip_get(ui)->last_shown = -1.0;
}

static void fpas_tooltip_tick(cui_ctx *ui) {
    if (ui->in_frame && ui->tooltip_id && ui->tooltip_id == ui->hot &&
        !ui->active && !ui->mdown[CUI_MOUSE_LEFT] &&
        ui->time - ui->hover_start < 0.55 && ui->wakeup > 0.2f) {
        /* A longer sleep would lose elapsed hover time to CUI's frame-delta cap. */
        cui__wakeup_after(ui, 0.2f);
    }
}

void fpas_cui_tooltip(cui_ctx *ui, const char *text, const char *shortcut) {
    if (!ui->last_id || ui->last_node < 0) return;
    const int anchor = fpas_find(ui, ui->last_id);
    if (anchor < 0 || !(ui->nodes[anchor].flags & CUI_NF_INTERACTIVE)) return;
    /* A composite leaf such as a slider exposes its track inside its layout root. */
    for (int node = anchor; node >= 0; node = ui->nodes[node].parent) {
        if (node == ui->last_node) {
            cui_tooltip(ui, shortcut ? cui__frame_printf(ui, "%s  \xc2\xb7  %s", text, shortcut) : text);
            return;
        }
    }
}

const char *fpas_cui_tooltip_inspect(cui_ctx *ui, int *node) {
    *node = fpas_find(ui, 0x7001718u);
    return *node >= 0 ? cui__tooltip_get(ui)->text : NULL;
}
