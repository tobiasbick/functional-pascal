// [Documentation](../../../../../docs/ui/gui/voice/voice-waveform.md)
#include "../../fpas_cui.h"

int fpas_cui_voice_wave(cui_ctx *ui, const char *id, const float *levels, const uint8_t *owners, int count,
    float *progress, int width_kind, float width, float pitch, float step, int owner, int palette, uint32_t rgba, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = progress && fpas_ai_enabled(ui, flags, key);
    const fpas_voice_wave_interaction input = fpas_voice_wave_interaction_begin(ui, key, enabled);
    const cui_voice_wave_opts opts = { .clip = (flags & 4) != 0, .progress = progress, .owners = owners,
        .owner = (cui_voice_owner)owner, .color = fpas_chart_color(palette, rgba), .pitch = pitch, .step = step,
        .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !enabled);
    bool changed = cui_voice_wave_ex(ui, id, levels, count, &opts);
    cui_pop_disabled(ui);
    changed |= fpas_voice_wave_interaction_end(ui, key, root, input, step, progress);
    if (!enabled) ui->nodes[root].flags &= ~(CUI_NF_INTERACTIVE | CUI_NF_FOCUSABLE);
    fpas_creative_node(ui, root, key);
    return enabled && changed;
}
