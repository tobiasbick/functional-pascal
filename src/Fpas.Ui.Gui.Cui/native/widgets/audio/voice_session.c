#include "../../fpas_cui.h"

int fpas_cui_voice_session(cui_ctx *ui, const char *id, int activity, const float *levels, const uint8_t *owners,
    int count, const char *label, const char *transcript, const char *upcoming, int speaker, const char *you,
    const char *assistant, int *muted, int width_kind, float width, int flags, int focus, int *nodes) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_push_id(ui, id);
    const cui_id parts[] = { cui_make_id(ui, "###mute"), cui_make_id(ui, "###end") };
    cui_pop_id(ui);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < 2) ui->focus_next_request = parts[focus];
    bool mute = muted && *muted;
    const cui_voice_session_opts opts = { .owners = owners, .label = label, .transcript = transcript,
        .upcoming = upcoming, .speaker = (cui_voice_owner)speaker, .you = you, .assistant = assistant,
        .muted = muted ? &mute : NULL, .no_end = (flags & 2) != 0, .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !enabled);
    const int result = cui_voice_session_ex(ui, id, (cui_voice_state)activity, levels, count, &opts);
    cui_pop_disabled(ui);
    for (int i = 0; i < 2; ++i) nodes[i] = fpas_find(ui, parts[i]);
    fpas_creative_node(ui, root, key);
    if (muted) *muted = mute;
    return enabled ? result : 0;
}
