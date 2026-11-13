static bool fpas_ai_enabled(cui_ctx *ui, int flags, cui_id key) {
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    return enabled;
}

/* CUI-034 (docs/future/cui-upstream-bugs.md#cui-034): keep the native
   running glyph still and preserve earlier animation requests. */
static void fpas_ai_freeze_status(cui_ctx *ui, int first, bool was_animating) {
    if (ui->theme.motion > 0) return;
    for (int node = first; node < ui->node_count; ++node) {
        if (ui->nodes[node].paint != cui__paint_status) continue;
        cui__st_paint *status = ui->nodes[node].paint_data;
        status->t = 0;
    }
    ui->animating = was_animating;
}

static void fpas_voice_stop_drag(cui_ctx *ui, cui_id key) {
    cui__vw_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (state) state->dragging = false;
}
