// [Documentation](../../../../../docs/ui/gui/voice/README.md)
static bool fpas_ai_enabled(cui_ctx *ui, int flags, cui_id key) {
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    return enabled;
}

static void fpas_voice_stop_drag(cui_ctx *ui, cui_id key) {
    cui__vw_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (state) state->dragging = false;
}

typedef struct {
    fpas_slider_release release;
    fpas_focus_cover cover;
    cui_interact input;
} fpas_voice_wave_interaction;

/** Keeps waveform painting and captures the mouse position at release. */
static fpas_voice_wave_interaction fpas_voice_wave_interaction_begin(cui_ctx *ui, cui_id key, bool enabled) {
    if (!enabled || !ui->window_focused) fpas_voice_stop_drag(ui, key);
    /* CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): use the final pointer position. */
    fpas_voice_wave_interaction saved = { .release = fpas_slider_release_begin(ui, key, enabled) };
    if (enabled && (ui->focus == key || ui->focus_next_request == key ||
        (ui->hot == key && ui->mpressed[CUI_MOUSE_LEFT]))) {
        saved.input = fpas_probe_drag_interaction(ui, key);
        /* CUI-099 (docs/future/cui-upstream-bugs.md#cui-099): preserve the waveform paint. */
        if (saved.input.focus_visible) saved.cover = fpas_begin_focus_cover(ui);
    }
    return saved;
}

/** Paints the native waveform and its keyboard focus ring. */
static void fpas_paint_focused_voice_wave(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_voice_wave(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

/** Applies native keyboard seek after protecting paint and restores physical pointer input. */
static bool fpas_voice_wave_interaction_end(cui_ctx *ui, cui_id key, int index,
    fpas_voice_wave_interaction saved, float step, float *progress) {
    bool changed = false;
    if (saved.input.focus_visible) {
        fpas_end_focus_cover(ui, saved.cover);
        cui_node *node = &ui->nodes[index];
        cui__vw_paint *paint = node->paint_data;
        if (saved.input.keyboard_focused) {
            const cui_interact keys = { .keyboard_focused = true };
            changed = cui__vw_seek(ui, key, keys, paint->pitch, step > 0 ? step : 0.05f, progress);
            if (changed) paint->progress = *progress >= 0 ? CUI_MIN(*progress, 1.0f) : 0;
        }
        paint->show_head = true;
        node->paint = fpas_paint_focused_voice_wave;
    }
    fpas_slider_release_end(ui, saved.release);
    if (saved.release.queued) {
        fpas_voice_stop_drag(ui, key);
        ui->mpressed[CUI_MOUSE_LEFT] = false;
        if (ui->active == key) ui->active = 0;
    }
    return changed;
}
