// [Documentation](../../../../docs/ui/gui/voice/voice-note.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

typedef struct {
    cui_ctx *ui;
    float duration, position, speed;
    int playing, parts[3];
} voice_note_case;

static int failed;

static void check(bool ok, const char *message) {
    if (ok) return;
    fprintf(stderr, "%s\n", message);
    ++failed;
}

static voice_note_case context(cui_direction direction) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .direction = direction });
    cui_set_viewport(ui, 680, 500, 1);
    cui_set_reduced_motion(ui, true);
    return (voice_note_case){ .ui = ui, .duration = 10 };
}

static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static int frame(voice_note_case *note, float available, int width_kind, float width,
    int flags, int focus, int popup) {
    static const float levels[] = { .2f, .5f, .8f, .6f, .9f, .2f, .3f, .6f, .5f };
    cui_ctx *ui = note->ui;
    int result = 0;
    cui_begin_frame(ui, .016f);
    cui_column(ui, .id = "note-container", .width = CUI_PX(available)) {
        if (popup) cui_button(ui, "Options");
        if (popup == 1) cui_popover_open(ui, "options");
        if (popup) cui_popover(ui, "options") { cui_text(ui, "Overlay"); }
        result = fpas_cui_voice_note(ui, "note", levels, sizeof levels / sizeof *levels,
            note->duration, &note->position, &note->speed, &note->playing,
            width_kind, width, flags, focus, note->parts);
        if (popup == 2)
            check(cui_key_pressed(ui, CUI_KEY_ENTER) || cui_key_pressed(ui, CUI_KEY_SPACE),
                "The voice note must leave the activation key available to the popover.");
    }
    cui_end_frame(ui);
    return result;
}

static int draw(voice_note_case *note, int flags, int focus) {
    return frame(note, 600, 0, 0, flags, focus, 0);
}

static cui_rect part(const voice_note_case *note, int index) {
    return fpas_cui_node_rect(note->ui, note->parts[index]);
}

static bool fits(cui_rect child, cui_rect box) {
    return child.w > 0 && child.h > 0 && child.x >= box.x && child.x + child.w <= box.x + box.w + .01f
        && child.y >= box.y && child.y + child.h <= box.y + box.h + .01f;
}

/** Checks native default sizing, compact wrapping, explicit widths, and the outgoing edge. */
static void responsive_note(void) {
    voice_note_case note = context(CUI_DIRECTION_LTR);
    const cui_id id = cui_make_id(note.ui, "note");
    const float widths[] = { 240, 600, 170, 96, 240 };
    const float expected[] = { 240, 330, 170, 96, 240 };
    for (int i = 0; i < 5; ++i) {
        frame(&note, widths[i], 0, 0, 3, -1, 0);
        const cui_rect r = cui_get_rect(note.ui, id);
        const cui_rect container = cui_get_rect(note.ui, cui_make_id(note.ui, "note-container"));
        check(r.w == expected[i] && r.x + r.w == container.x + container.w,
            "An outgoing note must fit the available width and align to the end.");
        for (int j = 0; j < 3; ++j)
            check(fits(part(&note, j), r), "The play, seek and speed controls must fit inside the note.");
        if (i == 3) check(r.h > 90, "The controls must wrap in a very narrow bubble.");
    }
    frame(&note, 600, 2, 160, 1, -1, 0);
    check(cui_get_rect(note.ui, id).w == 160, "A fixed width below 200 must be honored.");
    frame(&note, 600, 3, 50, 1, -1, 0);
    check(cui_get_rect(note.ui, id).w == 300, "The note must support percentage widths.");
    frame(&note, 600, 1, 0, 1, -1, 0);
    check(cui_get_rect(note.ui, id).w == 600, "Fill must use the full available width.");
    cui_destroy(note.ui);
}

static bool wave_ring(const voice_note_case *note, bool expect_dots) {
    const cui_rect wave = part(note, 1);
    const cui_draw_data *draw_data = cui_get_draw_data(note->ui);
    bool has_ring = false, has_dots = false;
    for (int i = 0; i < draw_data->vertex_count; ++i) {
        const cui_vertex *v = &draw_data->vertices[i];
        if (v->mode != CUI_SHADE_RRECT) continue;
        const float x = v->x - v->lx, y = v->y - v->ly;
        if (v->border == 2 && fabsf(x - wave.x - wave.w / 2) < .01f &&
            fabsf(y - wave.y - wave.h / 2) < .01f && fabsf(v->hw * 2 - wave.w - 6) < .01f)
            has_ring = true;
        if (v->hw > 1 && v->hw < 1.5f && v->hh == v->hw && x > wave.x && x < wave.x + wave.w &&
            y > wave.y && y < wave.y + wave.h) has_dots = true;
    }
    if (expect_dots) check(has_dots, "The waveform must keep its dots when focus changes.");
    return has_ring;
}

/** Checks both activation keys, speed wrap, keyboard seek and focus painting. */
static void transport_keys(void) {
    voice_note_case note = context(CUI_DIRECTION_RTL);
    draw(&note, 3, 0);
    const cui_rect play = part(&note, 0), wave = part(&note, 1), rate = part(&note, 2);
    check(play.x > wave.x && wave.x > rate.x, "Right-to-left layout must mirror the note's controls.");

    press(note.ui, CUI_KEY_ENTER);
    check(draw(&note, 1, -1) == CUI_PLAYER_TOGGLE && note.playing, "Enter must propose playback.");
    press(note.ui, CUI_KEY_SPACE);
    check(draw(&note, 1, -1) == CUI_PLAYER_TOGGLE && !note.playing, "Space must propose pausing.");
    note.position = 3;
    press(note.ui, CUI_KEY_RIGHT);
    check(draw(&note, 1, 1) == CUI_PLAYER_SEEK && fabsf(note.position - 4) < .001f,
        "Right must seek a second forward even in right-to-left layout.");
    check(wave_ring(&note, true), "A keyboard-focused waveform must draw its ring and its dots.");
    press(note.ui, CUI_KEY_LEFT);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && fabsf(note.position - 3) < .001f,
        "Left must seek a second backward.");
    press(note.ui, CUI_KEY_END);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && note.position == 10, "End must seek to the clip end.");
    press(note.ui, CUI_KEY_HOME);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && note.position == 0, "Home must seek to the clip start.");

    const cui_key keys[] = { CUI_KEY_SPACE, CUI_KEY_ENTER, CUI_KEY_SPACE };
    const float rates[] = { 1.5f, 2, 1 };
    for (int i = 0; i < 3; ++i) {
        press(note.ui, keys[i]);
        check(draw(&note, 1, 2) == CUI_PLAYER_SPEED && note.speed == rates[i],
            "The speed chip must advance from the default and cycle through all rates.");
    }
    check(!wave_ring(&note, true), "Moving focus must remove the old waveform ring.");
    cui_destroy(note.ui);
}

static void pointer(voice_note_case *note, cui_rect wave, float fraction, bool down) {
    cui_input_mouse_move(note->ui, wave.x + wave.w * fraction, wave.y + wave.h / 2);
    cui_input_mouse_button(note->ui, CUI_MOUSE_LEFT, down);
}

/** Tests drag releases coalesced with movement, and disables the note during an active seek. */
static void pointer_seek(void) {
    voice_note_case note = context(CUI_DIRECTION_LTR);
    draw(&note, 1, -1);
    const cui_rect wave = part(&note, 1);
    pointer(&note, wave, .2f, true);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && note.position < 2.1f, "The initial press must seek.");
    pointer(&note, wave, .8f, false);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && fabsf(note.position - 8) < .1f,
        "The final drag position must be applied even when movement arrives with the release.");
    pointer(&note, wave, .1f, true);
    pointer(&note, wave, .5f, false);
    check(draw(&note, 1, -1) == CUI_PLAYER_SEEK && fabsf(note.position - 5) < .1f,
        "A press and release in one frame must seek to the release position.");

    pointer(&note, wave, .1f, true);
    draw(&note, 1, -1);
    const float before = note.position;
    pointer(&note, wave, .6f, false);
    check(draw(&note, 0, -1) == 0 && note.position == before, "Disabling the note must cancel an active seek.");
    check(draw(&note, 1, -1) == 0 && note.position == before, "Re-enabling must not resume the cancelled seek.");
    cui_destroy(note.ui);
}

/** Keeps a zero-duration waveform visible but skips it in keyboard focus order. */
static void zero_duration(void) {
    voice_note_case note = context(CUI_DIRECTION_LTR);
    draw(&note, 1, 1);
    const cui_rect wave = part(&note, 1);
    note.duration = 0;
    pointer(&note, wave, .8f, true);
    check(draw(&note, 1, 1) == 0 && note.parts[1] == -1 && note.position == 0,
        "An empty note must not expose or activate seek after its duration becomes zero.");
    pointer(&note, wave, .8f, false);
    draw(&note, 1, -1);
    press(note.ui, CUI_KEY_TAB);
    draw(&note, 1, -1);
    check(cui_focused_id(note.ui) == fpas_cui_node_key(note.ui, note.parts[0]), "Tab must first focus Play.");
    press(note.ui, CUI_KEY_TAB);
    draw(&note, 1, -1);
    check(cui_focused_id(note.ui) == fpas_cui_node_key(note.ui, note.parts[2]),
        "Tab must skip the waveform of a zero-length recording.");
    cui_destroy(note.ui);
}

/** Keeps both neighboring activation keys with an open popover. */
static void covered_note(int focus) {
    voice_note_case note = context(CUI_DIRECTION_LTR);
    frame(&note, 500, 0, 0, 1, focus, 0);
    press(note.ui, CUI_KEY_ENTER);
    frame(&note, 500, 0, 0, 1, -1, 0);
    frame(&note, 500, 0, 0, 1, -1, 1);
    const float speed = note.speed;
    const int playing = note.playing;
    press(note.ui, CUI_KEY_ENTER);
    check(frame(&note, 500, 0, 0, 1, -1, 2) == 0, "Enter must stay with the popover.");
    press(note.ui, CUI_KEY_SPACE);
    check(frame(&note, 500, 0, 0, 1, -1, 2) == 0, "Space must stay with the popover.");
    check(note.speed == speed && note.playing == playing, "The covered note must preserve its transport state.");
    cui_destroy(note.ui);
}

/** Runs the voice note layout, painting, and input regressions. */
int main(void) {
    responsive_note();
    transport_keys();
    pointer_seek();
    zero_duration();
    covered_note(0);
    covered_note(2);
    return failed != 0;
}
