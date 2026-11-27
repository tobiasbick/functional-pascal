// [Documentation](../../../../docs/ui/gui/voice/voice-session.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

typedef struct {
    int flags, muted, nodes[2];
    bool show_mute, show_popup, key_preserved;
    const char *transcript, *upcoming;
    const uint8_t *owners;
} session_case;

static int failures;

/** Records a failed voice session assertion. */
static void check(bool value, const char *message) {
    if (value) return;
    fprintf(stderr, "%s\n", message);
    ++failures;
}

/** Creates a reduced-motion context with distinguishable speaker colors. */
static cui_ctx *context(cui_direction direction) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .accent = CUI_ACCENT_BLUE, .direction = direction });
    cui_set_viewport(ui, 620, 440, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

/** Renders the session, optionally injecting input and requesting part focus. */
static int frame(cui_ctx *ui, session_case *sample, cui_key key, int focus) {
    static const float levels[] = { 1, CUI_VOICE_SILENT, 1 };
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, .016f);
    cui_button(ui, "Popup anchor");
    if (sample->show_popup) cui_popover_open(ui, "Options");
    cui_popover(ui, "Options", .width = 150) { cui_text(ui, "Overlay"); }
    const int result = fpas_cui_voice_session(ui, "call", CUI_VOICE_SPEAKING, levels,
        sample->owners, 3, "Reply in progress", sample->transcript, sample->upcoming,
        CUI_VOICE_REPLY, "Caller", "Guide", sample->show_mute ? &sample->muted : NULL,
        2, 300, sample->flags, focus, sample->nodes);
    sample->key_preserved = key != CUI_KEY_NONE && cui_key_pressed(ui, key);
    cui_end_frame(ui);
    return result;
}

/** Looks up a session part using the same scope as the native widget. */
static cui_id part(cui_ctx *ui, const char *name) {
    cui_push_id(ui, "call");
    const cui_id key = cui_make_id(ui, name);
    cui_pop_id(ui);
    return key;
}

/** Tests both call actions and the focus lifecycle for hidden and disabled buttons. */
static void controls(void) {
    cui_ctx *ui = context(CUI_DIRECTION_LTR);
    session_case sample = { .flags = 1, .show_mute = true };
    frame(ui, &sample, CUI_KEY_NONE, 0);
    check(sample.nodes[0] >= 0 && sample.nodes[1] >= 0, "Mute and End must be visible by default.");
    check(cui_focused_id(ui) == part(ui, "###mute"), "The adapter must focus the requested Mute button.");
    check(frame(ui, &sample, CUI_KEY_ENTER, -1) == CUI_VOICE_SESSION_MUTE && sample.muted,
        "Enter must report the proposed microphone state.");
    check(frame(ui, &sample, CUI_KEY_SPACE, -1) == CUI_VOICE_SESSION_MUTE && !sample.muted,
        "Mute must retain focus after its caption changes to Unmute.");
    frame(ui, &sample, CUI_KEY_TAB, -1);
    check(cui_focused_id(ui) == part(ui, "###end"), "Tab must move from Mute to End.");
    check(frame(ui, &sample, CUI_KEY_ENTER, -1) == CUI_VOICE_SESSION_END && !sample.muted,
        "End must report its own action and leave mute state unchanged.");

    sample.flags = 3;
    frame(ui, &sample, CUI_KEY_NONE, 1);
    check(sample.nodes[1] < 0 && cui_focused_id(ui) != part(ui, "###end"),
        "A hidden End button must release focus and ignore focus requests.");
    sample.flags = 1;
    check(frame(ui, &sample, CUI_KEY_SPACE, -1) == 0,
        "Restoring End must leave it inactive until it receives focus.");

    frame(ui, &sample, CUI_KEY_NONE, 0);
    sample.show_mute = false;
    frame(ui, &sample, CUI_KEY_NONE, 0);
    check(sample.nodes[0] < 0 && cui_focused_id(ui) != part(ui, "###mute"),
        "Omitting the mute state must omit the button and clear its focus.");
    sample.show_mute = true;
    check(frame(ui, &sample, CUI_KEY_ENTER, -1) == 0 && !sample.muted,
        "Restoring Mute must leave it inactive until it receives focus.");

    frame(ui, &sample, CUI_KEY_NONE, 0);
    sample.flags = 0;
    check(frame(ui, &sample, CUI_KEY_SPACE, 0) == 0 && !sample.muted && sample.nodes[0] >= 0,
        "A disabled Mute button must stay visible and ignore activation.");
    sample.flags = 1;
    check(frame(ui, &sample, CUI_KEY_ENTER, -1) == 0 && !sample.muted,
        "Re-enabling the session must leave focus released.");
    cui_destroy(ui);
}

/** Tests that an open popover owns activation keys while the session is covered. */
static void covered_controls(void) {
    cui_ctx *ui = context(CUI_DIRECTION_LTR);
    session_case sample = { .flags = 1, .show_mute = true };
    frame(ui, &sample, CUI_KEY_NONE, 0);
    frame(ui, &sample, CUI_KEY_ENTER, -1);
    sample.show_popup = true;
    frame(ui, &sample, CUI_KEY_NONE, -1);
    check(frame(ui, &sample, CUI_KEY_ENTER, -1) == 0 && sample.muted && sample.key_preserved,
        "A covered Mute button must leave Enter to the open popover.");
    check(frame(ui, &sample, CUI_KEY_SPACE, -1) == 0 && sample.muted && sample.key_preserved,
        "A covered Mute button must leave Space to the open popover.");
    cui_destroy(ui);
}

/** Finds the painted waveform dot at a column and row center. */
static cui_color dot_at(const cui_draw_data *draw, float x, float y) {
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_RRECT && fabsf(v->x - v->lx - x) < .1f &&
            fabsf(v->y - v->ly - y) < .1f && v->hw < 2) return v->color;
    }
    return (cui_color){ 0 };
}

/** Tests display-only signal order, speaker colors and an upcoming-only transcript. */
static void timeline(void) {
    static const uint8_t owners[] = { CUI_VOICE_YOU, CUI_VOICE_REPLY, CUI_VOICE_REPLY };
    cui_ctx *ui = context(CUI_DIRECTION_RTL);
    session_case sample = { .flags = 3, .owners = owners };
    frame(ui, &sample, CUI_KEY_NONE, -1);
    const cui_id call = cui_make_id(ui, "call");
    const cui_rect initial = cui_get_rect(ui, call);
    sample.upcoming = "Remainder of the speaker's turn";
    frame(ui, &sample, CUI_KEY_NONE, -1);
    const cui_rect root = cui_get_rect(ui, call);
    const cui_rect wave = cui_get_rect(ui, part(ui, "wave"));
    check(root.w == 300 && wave.w == root.w, "The timeline must fill the requested session width.");
    check(root.h > initial.h + 25, "Upcoming-only transcript text must reserve transcript space.");
    check(sample.nodes[0] < 0 && sample.nodes[1] < 0, "The session can display without either action.");

    const float pitch = 3.6f, columns = floorf(wave.w / pitch);
    const float right = wave.x + (wave.w + columns * pitch) * .5f;
    const float row = wave.y + wave.h / 2 - 2 * pitch;
    const cui_draw_data *draw = cui_get_draw_data(ui);
    const cui_color local = dot_at(draw, right - 2.5f * pitch, row);
    const cui_color silent = dot_at(draw, right - 1.5f * pitch, row);
    const cui_color reply = dot_at(draw, right - .5f * pitch, row);
    check(local.a > 0 && silent.a == 0 && reply.a > 0 && reply.b > reply.r,
        "The tagged timeline must draw local speech, a flat gap and the accent reply left to right.");

    frame(ui, &sample, CUI_KEY_TAB, -1);
    check(cui_focused_id(ui) == cui_make_id(ui, "Popup anchor"),
        "Tab must skip the display-only timeline.");
    cui_destroy(ui);
}

/** Runs the voice session regression checks. */
int main(void) {
    controls();
    covered_controls();
    timeline();
    return failures != 0;
}
