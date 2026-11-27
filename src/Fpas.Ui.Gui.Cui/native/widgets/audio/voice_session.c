// [Documentation](../../../../../docs/ui/gui/voice/voice-session.md)
#include "../../fpas_cui.h"

/** Draws the voice session and reports actions from its available controls. */
int fpas_cui_voice_session(cui_ctx *ui, const char *id, int activity, const float *levels, const uint8_t *owners,
    int count, const char *label, const char *transcript, const char *upcoming, int speaker, const char *you,
    const char *assistant, int *muted, int width_kind, float width, int flags, int focus, int *nodes) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_push_id(ui, id);
    const cui_id parts[] = { cui_make_id(ui, "###mute"), cui_make_id(ui, "###end") };
    cui_pop_id(ui);
    const bool visible[] = { muted != NULL, !(flags & 2) };
    // CUI-049 (docs/future/cui-upstream-bugs.md#cui-049): a removed button must release focus.
    fpas_own(ui, key, visible[0] ? parts[0] : 0, visible[1] ? parts[1] : 0, 0);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    for (int i = 0; i < 2; ++i) {
        if (ui->focus_next_request == parts[i] && (!enabled || !visible[i])) ui->focus_next_request = 0;
    }
    if (enabled && focus >= 0 && focus < 2 && visible[focus]) ui->focus_next_request = parts[focus];
    bool mute = muted && *muted;
    const cui_voice_session_opts opts = { .owners = owners, .label = label, .transcript = transcript,
        .upcoming = upcoming, .speaker = (cui_voice_owner)speaker, .you = you, .assistant = assistant,
        .muted = muted ? &mute : NULL, .no_end = !visible[1], .width = fpas_extent(width_kind, width) };

    // CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): give activation keys to the open popup.
    const bool covered = enabled && cui__keys_covered(ui);
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    cui_push_disabled(ui, !enabled);
    const int result = cui_voice_session_ex(ui, id, (cui_voice_state)activity, levels, count, &opts);
    cui_pop_disabled(ui);
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }
    for (int i = 0; i < 2; ++i) nodes[i] = visible[i] ? fpas_find(ui, parts[i]) : -1;
    fpas_creative_node(ui, root, key);
    if (muted) *muted = mute;
    return enabled ? result : 0;
}
