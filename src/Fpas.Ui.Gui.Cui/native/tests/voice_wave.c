// [Documentation](../../../../docs/ui/gui/voice/voice-waveform.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

typedef struct {
    float progress;
    int flags;
    bool popup;
    bool clicked;
} wave_case;

static bool wave_frame(cui_ctx *ui, wave_case *wave) {
    const float levels[] = {0.4f, 0.9f, CUI_VOICE_SILENT, 0.7f};
    cui_begin_frame(ui, .016f);
    const int changed = fpas_cui_voice_wave(ui, "wave", levels, NULL, 4, &wave->progress,
        2, 120, 10, .2f, CUI_VOICE_YOU, 0, 0, wave->flags);
    wave->clicked = cui_button(ui, "Other control");
    if (wave->popup) {
        cui_button(ui, "Options");
        cui_popover_open(ui, "options");
        if (cui_popover_begin(ui, "options", NULL)) {
            cui_text(ui, "Wave options");
            cui_popover_end(ui);
        }
    }
    cui_end_frame(ui);
    return changed != 0;
}

static void wave_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static void wave_mouse(cui_ctx *ui, cui_rect bounds, float fraction) {
    cui_input_mouse_move(ui, bounds.x + bounds.w * fraction, bounds.y + bounds.h * .5f);
}

/** Checks press and release in one frame, and a final movement arriving with release. */
static bool wave_pointer(cui_ctx *ui) {
    wave_case wave = { .progress = .1f, .flags = 1 };
    wave_frame(ui, &wave);
    const cui_rect bounds = cui_get_rect(ui, cui_make_id(ui, "wave"));
    wave_mouse(ui, bounds, .5f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    bool ok = wave_frame(ui, &wave) && fabsf(wave.progress - .5f) < .001f;

    const cui_rect other = cui_get_rect(ui, cui_make_id(ui, "Other control"));
    wave_mouse(ui, bounds, .2f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    wave_mouse(ui, other, .5f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    ok &= wave_frame(ui, &wave) && !wave.clicked;

    wave_mouse(ui, bounds, .2f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    ok &= wave_frame(ui, &wave) && fabsf(wave.progress - .2f) < .001f;
    wave_mouse(ui, bounds, .8f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    ok &= wave_frame(ui, &wave) && fabsf(wave.progress - .8f) < .001f;

    wave_mouse(ui, other, .5f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    ok &= !wave_frame(ui, &wave) && wave.clicked && fabsf(wave.progress - .8f) < .001f;
    if (!ok) fprintf(stderr, "Voice seeks lost a coalesced click or release position.\n");
    return ok;
}

/** Checks that keyboard focus preserves dots and seeks only while the strip owns input. */
static bool wave_keyboard(cui_ctx *ui) {
    wave_case wave = { .progress = .4f, .flags = 1 };
    cui_input_mouse_leave(ui);
    wave_frame(ui, &wave);
    wave_frame(ui, &wave);
    const cui_id id = cui_make_id(ui, "wave");
    const int normal = cui_get_draw_data(ui)->vertex_count;
    wave_key(ui, CUI_KEY_TAB);
    wave_frame(ui, &wave);
    bool ok = cui_focused_id(ui) == id && cui_get_draw_data(ui)->vertex_count > normal;
    wave_key(ui, CUI_KEY_RIGHT);
    ok &= wave_frame(ui, &wave) && fabsf(wave.progress - .6f) < .001f;
    wave_key(ui, CUI_KEY_LEFT);
    ok &= wave_frame(ui, &wave) && fabsf(wave.progress - .4f) < .001f;
    wave_key(ui, CUI_KEY_END);
    ok &= wave_frame(ui, &wave) && wave.progress == 1;
    wave_key(ui, CUI_KEY_HOME);
    ok &= wave_frame(ui, &wave) && wave.progress == 0;

    wave.popup = true;
    wave_frame(ui, &wave);
    wave_key(ui, CUI_KEY_END);
    ok &= !wave_frame(ui, &wave) && wave.progress == 0;

    wave.flags = 0;
    wave.popup = false;
    wave_key(ui, CUI_KEY_RIGHT);
    ok &= !wave_frame(ui, &wave) && wave.progress == 0 && cui_focused_id(ui) != id;
    if (!ok) fprintf(stderr, "Voice focus lost the dots, keyboard seeks or popup ownership.\n");
    return ok;
}

int main(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 320, 220, 1);
    cui_set_reduced_motion(ui, true);
    const bool keyboard = wave_keyboard(ui);
    cui_destroy(ui);

    ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 320, 220, 1);
    cui_set_reduced_motion(ui, true);
    const bool pointer = wave_pointer(ui);
    cui_destroy(ui);
    return keyboard && pointer ? 0 : 1;
}
