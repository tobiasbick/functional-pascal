// [Documentation](../../../../../docs/ui/gui/motion/typewriter.md)
#include "../../fpas_cui.h"

/** Draws the native typewriter and reports its completion state and transition. */
int fpas_cui_typewriter(cui_ctx *ui, const char *id, const char *text, int replay, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_motion_replay(ui, key, replay);
    const cui__tw_state *previous = cui__state_lazy(ui, key, sizeof *previous, false);
    const size_t length = strlen(text);
    const bool was_done = previous && previous->init && previous->bytes >= previous->prev_len &&
        (size_t)previous->prev_len == length && previous->prev_hash == cui__hash(text, length, 0x7E1u);
    const cui_text_fx_opts options = fpas_motion_options(o);
    const bool done = cui_typewriter_ex(ui, id, text, &options);
    // CUI-097 (docs/future/cui-upstream-bugs.md#cui-097): blink the native caret until completion.
    if (!done && options.caret) {
        const cui__tw_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        cui__tw_fx *paint = ui->nodes[node].text_fx_data;
        const double phase = fmod(cui_time(ui) - state->started, 1.0);
        paint->caret = phase < 0.5;
        if (!ui->animating) cui__wakeup_after(ui, (float)((paint->caret ? 0.5 : 1.0) - phase));
    }
    fpas_motion_node(ui, node, key);
    return done ? (was_done ? 1 : 3) : 0;
}
