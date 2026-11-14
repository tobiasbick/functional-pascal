static bool fpas_ai_enabled(cui_ctx *ui, int flags, cui_id key) {
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    return enabled;
}

static void fpas_voice_stop_drag(cui_ctx *ui, cui_id key) {
    cui__vw_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (state) state->dragging = false;
}
