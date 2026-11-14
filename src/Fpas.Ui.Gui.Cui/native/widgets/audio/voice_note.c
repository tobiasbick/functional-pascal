#include "../../fpas_cui.h"

int fpas_cui_voice_note(cui_ctx *ui, const char *id, const float *levels, int count, float duration,
    float *position, float *speed, int *playing, int width_kind, float width, int flags, int focus, int *nodes) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), play = key ^ 0x9A1Bu, rate = key ^ 0x5BEEu;
    cui_push_id(ui, id);
    const cui_id wave = cui_make_id(ui, "wave");
    cui_pop_id(ui);
    const cui_id parts[] = { play, wave, rate };
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (!enabled) fpas_voice_stop_drag(ui, wave);
    if (enabled && focus >= 0 && focus < 3) ui->focus_next_request = parts[focus];
    fpas_own(ui, key, wave, rate, 0);
    cui_voice_note note = { levels, count, *position, duration, *speed, *playing != 0 };
    const cui_voice_note_opts opts = { .mine = (flags & 2) != 0, .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !enabled);
    const int result = cui_voice_note_ex(ui, id, &note, &opts);
    cui_pop_disabled(ui);
    for (int i = 0; i < 3; ++i) nodes[i] = fpas_find(ui, parts[i]);
    fpas_creative_node(ui, root, key);
    *position = note.position;
    *speed = note.speed;
    *playing = note.playing;
    return enabled ? result : 0;
}
