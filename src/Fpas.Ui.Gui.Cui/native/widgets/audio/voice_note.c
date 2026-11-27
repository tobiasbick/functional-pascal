// [Documentation](../../../../../docs/ui/gui/voice/voice-note.md)
#include "../../fpas_cui.h"

/** CUI-172 (docs/future/cui-upstream-bugs.md#cui-172): keep a zero-length waveform visible and passive. */
static void fpas_voice_note_passive_wave(cui_ctx *ui, int wave_node, cui_id wave, int focus_start) {
    cui_node *node = &ui->nodes[wave_node];
    node->flags &= ~(CUI_NF_INTERACTIVE | CUI_NF_FOCUSABLE);
    for (int i = focus_start; i < ui->focus_count; ++i) {
        if (ui->focus_order[i].id != wave) continue;
        --ui->focus_count;
        memmove(ui->focus_order + i, ui->focus_order + i + 1,
            (size_t)(ui->focus_count - i) * sizeof *ui->focus_order);
        break;
    }
}

/** Draws the native note and reports valid transport proposals. */
int fpas_cui_voice_note(cui_ctx *ui, const char *id, const float *levels, int count, float duration,
    float *position, float *speed, int *playing, int width_kind, float width, int flags, int focus, int *nodes) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), play = key ^ 0x9A1Bu, rate = key ^ 0x5BEEu;
    cui_push_id(ui, id);
    const cui_id wave = cui_make_id(ui, "wave");
    cui_pop_id(ui);
    const cui_id parts[] = { play, wave, rate };
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool seekable = duration > 0;
    const bool covered = enabled && cui__keys_covered(ui);
    if (enabled && !covered && focus >= 0 && focus < 3 && (focus != 1 || seekable)) {
        /* The waveform's paint guard must not mask activation of the newly focused control. */
        if (ui->focus == wave && focus != 1) ui->focus = parts[focus];
        ui->focus_next_request = parts[focus];
    }
    const cui_id hot = ui->hot;
    const int focus_start = ui->focus_count;
    if (!seekable) {
        fpas_clear_interaction(ui, wave);
        if (hot == wave) ui->hot = 0;
    }
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): preserve the popup's activation keys. */
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    const fpas_voice_wave_interaction seek = fpas_voice_wave_interaction_begin(ui, wave, enabled && seekable);
    fpas_own(ui, key, wave, rate, 0);
    cui_voice_note note = { levels, count, *position, duration, *speed, *playing != 0 };
    const cui_voice_note_opts opts = { .mine = (flags & 2) != 0, .width = fpas_extent(width_kind, width) };
    cui_push_disabled(ui, !enabled);
    int result = cui_voice_note_ex(ui, id, &note, &opts);
    cui_pop_disabled(ui);
    for (int i = 0; i < 3; ++i) nodes[i] = fpas_find(ui, parts[i]);
    float progress = seekable ? cui__clampf(note.position / duration, 0, 1) : 0;
    const float step = seekable ? CUI_MIN(1.0f, 1.0f / duration) : 0;
    if (fpas_voice_wave_interaction_end(ui, wave, nodes[1], seek, step, &progress)) {
        note.position = progress * duration;
        const int meta = ui->nodes[nodes[1]].next;
        fpas_text(ui, fpas_label_node(ui, meta),
            cui__frame_printf(ui, "%s / %s", cui__mmss(ui, note.position), cui__mmss(ui, duration)));
        result |= CUI_PLAYER_SEEK;
    }
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }

    ui->nodes[nodes[1]].min_w = 6 * CUI__VW_PITCH;
    if (!seekable) {
        ui->hot = hot;
        fpas_voice_note_passive_wave(ui, nodes[1], wave, focus_start);
        nodes[1] = -1;
        result &= ~CUI_PLAYER_SEEK;
    }
    /* CUI-171 (docs/future/cui-upstream-bugs.md#cui-171): let a narrow bubble wrap its controls. */
    ui->nodes[root].min_w = 0;
    ui->nodes[root].flags |= CUI_NF_ROW_WRAP;
    fpas_creative_node(ui, root, key);
    *position = note.position;
    *speed = note.speed;
    *playing = note.playing;
    return enabled ? result : 0;
}
