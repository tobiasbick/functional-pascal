#include "../fpas_cui.h"

int fpas_cui_voice_orb(cui_ctx *ui, const char *id, int activity, float level, float size, int palette, uint32_t rgba, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    const cui_voice_orb_opts opts = { .size = size, .color = fpas_chart_color(palette, rgba) };
    cui_push_disabled(ui, !enabled);
    const bool clicked = cui_voice_orb_ex(ui, id, (cui_voice_state)activity, level, &opts);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    return enabled && clicked;
}
