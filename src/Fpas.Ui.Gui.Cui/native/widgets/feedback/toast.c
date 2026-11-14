#include "../../fpas_cui.h"

static const cui_id fpas_toast_layer = 0x70A57u;

int fpas_cui_toast(cui_ctx *ui, const char *title, const char *description,
    int tone, float duration, const char *action, int timer) {
    const cui_toast_opts options = {
        .description = description, .tone = (cui_tone)tone, .duration = duration,
        .action = action, .show_timer = timer != 0
    };
    return cui_toast_ex(ui, title, &options);
}

int fpas_cui_toast_inspect(cui_ctx *ui, int handle, int *card, int *action, int *dismiss) {
    *card = *action = *dismiss = -1;
    const int invoked = cui_toast_action_clicked(ui, handle) ? 2 : 0;
    for (int i = 0; i < ui->toast_count; ++i) {
        const cui_toast_item *toast = &ui->toasts[i];
        if (toast->handle != handle) continue;
        if (toast->dismissed) return invoked;
        const int node = fpas_find(ui, cui__hash(&handle, sizeof handle, fpas_toast_layer));
        if (node < 0 || ui->nodes[node].opacity <= 0.01f) return 1 | invoked;
        *card = node;
        const int close = ui->nodes[node].last;
        if (close >= 0 && ui->nodes[close].opacity > 0.01f) *dismiss = close;
        if (toast->action[0]) {
            for (int child = ui->nodes[node].first; child >= 0; child = ui->nodes[child].next) {
                if (ui->nodes[child].next == close && ui->nodes[child].opacity > 0.01f) {
                    *action = child;
                    break;
                }
            }
        }
        return 1 | invoked;
    }
    return invoked;
}
