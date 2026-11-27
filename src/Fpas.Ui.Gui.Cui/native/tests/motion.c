// [Documentation](../../../../docs/ui/gui/motion/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static bool gradient_endpoints(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 450, 300, 1);
    cui_set_reduced_motion(ui, true);
    const fpas_motion_text text = { .size = 36, .palette = -1, .rgba = 0xff0000ffu };
    bool valid = true;
    for (int animated = 0; animated <= 1; ++animated) {
        cui_begin_frame(ui, .016f);
        cui_row(ui, .align = CUI_ALIGN_START) {
            fpas_cui_gradient_text(ui, "gradient", "MMMMMMMMMM", &text, -1, 0xffff0000u, animated);
        }
        cui_end_frame(ui);
        const cui_draw_data *draw = cui_get_draw_data(ui);
        const cui_vertex *left = NULL, *right = NULL;
        for (int index = 0; index < draw->vertex_count; ++index) {
            const cui_vertex *glyph = &draw->vertices[index];
            if (glyph->mode != CUI_SHADE_TEXT || !glyph->color.a) continue;
            if (!left || glyph->x < left->x) left = glyph;
            if (!right || glyph->x > right->x) right = glyph;
        }
        valid &= left && right && left->color.r > left->color.b &&
            (animated ? right->color.r > right->color.b : right->color.b > right->color.r);
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Gradient text lost its endpoints or frozen animated phase.\n");
    return valid;
}

static int spotlight_frame(cui_ctx *ui, int flags, bool disabled, bool request) {
    cui_begin_frame(ui, .016f);
    if (request) cui_request_redraw(ui);
    cui_push_disabled(ui, disabled);
    const int node = fpas_cui_spotlight_begin(ui, "spot", 2, 240, 2, 100, 0, 0, 0, 0, flags);
    cui_text(ui, "Card");
    fpas_cui_spotlight_end(ui, node);
    cui_pop_disabled(ui);
    cui_end_frame(ui);
    return cui_get_draw_data(ui)->vertex_count;
}

static bool spotlight_hover_contract(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 450, 300, 1);
    cui_set_reduced_motion(ui, true);
    cui_input_mouse_leave(ui);
    const int idle = spotlight_frame(ui, 2, false, false);
    cui_input_mouse_move(ui, 40, 30);
    bool valid = spotlight_frame(ui, 0, false, false) == idle &&
        spotlight_frame(ui, 2, true, false) == idle;
    int hover = 0;
    for (int frame = 0; frame < 6; ++frame) hover = spotlight_frame(ui, 3, false, false);
    valid &= hover > idle && !cui_needs_redraw(ui);
    spotlight_frame(ui, 2, false, true);
    valid &= cui_needs_redraw(ui);
    cui_set_reduced_motion(ui, false);
    spotlight_frame(ui, 2, false, false);
    valid &= !cui_needs_redraw(ui);
    spotlight_frame(ui, 3, false, false);
    valid &= cui_needs_redraw(ui);
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Spotlight input or redraw ownership differs from its contract.\n");
    return valid;
}

static int control_tile_frame(cui_ctx *ui, const char *icon, const char *detail, int *on, int flags) {
    cui_begin_frame(ui, .016f);
    fpas_cui_control_tile(ui, "tile", "Settings", icon, detail, on, 2, 240, 0, 0, flags);
    cui_end_frame(ui);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    int glyphs = 0;
    for (int index = 0; index < draw->vertex_count; ++index)
        if (draw->vertices[index].mode == CUI_SHADE_TEXT) glyphs++;
    return glyphs;
}

static bool control_tile_contract(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 450, 300, 1);
    cui_set_reduced_motion(ui, true);
    int on = 0;
    const int defaults = control_tile_frame(ui, NULL, NULL, &on, 7);
    const int empty = control_tile_frame(ui, "", "", &on, 3);
    key(ui, CUI_KEY_SPACE);
    control_tile_frame(ui, "", "", &on, 3);
    bool valid = defaults > empty && on == 1;
    key(ui, CUI_KEY_ENTER);
    control_tile_frame(ui, "", "", &on, 1);
    valid &= on == 1;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Control tile lost native defaults, empty text or input gating.\n");
    return valid;
}

static bool motion_cards_contract(void) {
    return gradient_endpoints() && spotlight_hover_contract() && control_tile_contract();
}

static bool tabs(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    const cui_tab_bar_item items[] = {
        { CUI_ICON_HOUSE, "Home", 0 }, { CUI_ICON_SEARCH, "Search", 0 },
        { CUI_ICON_INBOX, "Inbox", 4 }, { CUI_ICON_USER, "Profile", 0 }
    };
    const char *icons[4], *labels[4];
    int badges[4], parts[4], selected = 0;
    for (int i = 0; i < 4; ++i) {
        icons[i] = items[i].icon; labels[i] = items[i].label; badges[i] = items[i].badge;
    }
    const cui_key input[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_RIGHT, CUI_KEY_LEFT };
    const int expected[] = { 0, 0, 1, 0 };
    bool ok = true;
    for (int i = 0; i < 4; ++i) {
        if (i) key(ui, input[i]);
        cui_begin_frame(ui, .016f);
        const bool changed = adapter
            ? fpas_cui_tab_bar(ui, "tabs", icons, labels, badges, 4, &selected, 0, 0, 2, -1, parts)
            : cui_tab_bar(ui, "tabs", items, 4, &selected);
        cui_end_frame(ui);
        if (selected != expected[i] || changed != (i >= 2)) {
            fprintf(stderr, "%s tab frame %d: selected=%d, changed=%d\n",
                adapter ? "FPAS" : "CUI", i, selected, changed);
            ok = false;
        }
        if (adapter && i && !fpas_cui_focused(ui, parts[expected[i]])) ok = false;
    }
    cui_destroy(ui);
    return ok;
}

static bool same_motion_vertices(cui_ctx *a, cui_ctx *b) {
    const cui_draw_data *first = cui_get_draw_data(a), *second = cui_get_draw_data(b);
    return first->vertex_count == second->vertex_count &&
        !memcmp(first->vertices, second->vertices, first->vertex_count * sizeof *first->vertices);
}

static bool tab_presentation(void) {
    cui_ctx *native = cui_create(NULL), *adapter = cui_create(NULL);
    cui_set_viewport(native, 420, 120, 1); cui_set_viewport(adapter, 420, 120, 1);
    cui_set_reduced_motion(native, true); cui_set_reduced_motion(adapter, true);
    const char *icons[] = {NULL, "", CUI_ICON_INBOX}, *labels[] = {"Circle", NULL, ""};
    const int badges[] = {0, 5, 42};
    const cui_tab_bar_item items[] = {{icons[0], labels[0], badges[0]},
        {icons[1], labels[1], badges[1]}, {icons[2], labels[2], badges[2]}};
    int parts[3];
    bool ok = true;
    for (int frame = 0; frame < 6; ++frame) {
        int expected = frame / 2 - 1, actual = expected;
        cui_begin_frame(native, .04f); cui_begin_frame(adapter, .04f);
        cui_tab_bar(native, "tabs", items, 3, &expected, .width=CUI_PX(260), .labels_on_selected_only=frame < 4);
        fpas_cui_tab_bar(adapter, "tabs", icons, labels, badges, 3, &actual, 2, 260, frame < 4 ? 3 : 2, -1, parts);
        cui_end_frame(native); cui_end_frame(adapter);
        ok &= actual == expected && same_motion_vertices(native, adapter);
    }
    cui_destroy(native); cui_destroy(adapter);
    if (!ok) fprintf(stderr, "Tab Bar presentation differs from native options\n");
    return ok;
}

static bool tab_resize(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 160, 1); cui_set_reduced_motion(ui, true);
    const char *icons[] = {CUI_ICON_HOUSE, CUI_ICON_SEARCH, CUI_ICON_INBOX};
    const char *labels[] = {"Home", "Search", "Inbox"};
    int badges[] = {0, 0, 4}, selected = 2, parts[3];
    bool ok = true;
    for (int frame = 0; frame < 8; ++frame) {
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width=CUI_PX(frame >= 3 && frame <= 5 ? 640 : 320)) {
            fpas_cui_tab_bar(ui, "tabs", icons, labels, badges, 3, &selected, 1, 0, 2, -1, parts);
        }
        cui_end_frame(ui);
        if (frame >= 2) ok &= cui_needs_redraw(ui) == (frame == 3 || frame == 6);
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Tab Bar must reconcile the pill when its parent width changes\n");
    return ok;
}

static bool dock_presentation(void) {
    cui_ctx *native = cui_create(NULL), *adapter = cui_create(NULL);
    cui_set_viewport(native, 640, 160, 1); cui_set_viewport(adapter, 640, 160, 1);
    cui_set_reduced_motion(native, true); cui_set_reduced_motion(adapter, true);
    const char *icons[] = { CUI_ICON_HOUSE, NULL, NULL }, *labels[] = { "Home", NULL, NULL };
    const int palettes[] = {-1, 0, 0}, kinds[] = {2, 0, 1}, badges[] = {150, 0, 0};
    const uint32_t colors[] = {0xffaa784du, 0, 0};
    int parts[3];
    cui_begin_frame(native, .04f); cui_begin_frame(adapter, .04f);
    cui_dock(native, "dock", .size=39, .magnify=1.3f) {
        cui_dock_item(native, icons[0], labels[0], .color=CUI_RGB(0x4d78aa), .running=true, .badge=150);
        cui_dock_separator(native);
        cui_dock_item(native, NULL, NULL);
    }
    fpas_cui_dock(adapter, "dock", icons, labels, palettes, colors, badges, kinds, 3, 39, 1.3f, 1, -1, parts);
    cui_end_frame(native); cui_end_frame(adapter);
    bool ok = same_motion_vertices(native, adapter) && parts[0] >= 0 && parts[1] < 0 && parts[2] >= 0;
    cui_destroy(native); cui_destroy(adapter);

    adapter = cui_create(NULL);
    cui_set_viewport(adapter, 300, 160, 1); cui_set_reduced_motion(adapter, true);
    const char *compact_icons[8] = {0}, *compact_labels[8] = {0};
    const int compact_badges[8] = {0}, compact_palettes[8] = {0}, compact_kinds[8] = {1,1,1,1,1,1,1,1};
    const uint32_t compact_colors[8] = {0};
    int compact_parts[8];
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2) cui_input_mouse_move(adapter, 18, 6);
        cui_begin_frame(adapter, .04f);
        fpas_cui_dock(adapter, "compact", compact_icons, compact_labels, compact_palettes, compact_colors,
            compact_badges, compact_kinds, 8, 0, 0, 1, -1, compact_parts);
        cui_end_frame(adapter);
        ok &= cui_get_rect(adapter, cui_make_id(adapter, "compact")).w == 268;
        if (frame >= 4) ok &= !cui_needs_redraw(adapter);
    }
    cui_destroy(adapter);
    if (!ok) fprintf(stderr, "Dock options, initial compact size or idle scheduling differ\n");
    return ok;
}

static bool navigation_retirement(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 300, 1);
    const char *values[] = { NULL, NULL };
    const int badges[] = {0, 0}, kinds[] = {1, 1};
    const uint32_t colors[] = {0, 0};
    cui_id retired_dock = 0, retired_tab = 0, first_dock = 0, first_tab = 0;
    for (int count = 2; count > 0; --count) {
        int dock[2], tabs[2], selected = -1;
        cui_begin_frame(ui, .04f);
        fpas_cui_dock(ui, "dock", values, values, badges, colors, badges, kinds, count, 0, 0, 1, -1, dock);
        fpas_cui_tab_bar(ui, "tabs", values, values, badges, count, &selected, 1, 0, 2, -1, tabs);
        if (count == 2) {
            retired_dock = fpas_cui_node_key(ui, dock[1]);
            retired_tab = fpas_cui_node_key(ui, tabs[1]);
        }
        first_dock = fpas_cui_node_key(ui, dock[0]);
        first_tab = fpas_cui_node_key(ui, tabs[0]);
        cui_end_frame(ui);
    }
    bool ok = retired_dock && retired_tab && !cui_get_rect(ui, retired_dock).w && !cui_get_rect(ui, retired_tab).w;
    fpas_cui_forget(ui, cui_make_id(ui, "dock")); fpas_cui_forget(ui, cui_make_id(ui, "tabs"));
    ok &= first_dock && first_tab && !cui_get_rect(ui, first_dock).w && !cui_get_rect(ui, first_tab).w;
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Navigation entries retained native state after removal\n");
    return ok;
}

static bool typewriter(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    const fpas_motion_text options = { .speed = 32 };
    const float elapsed[] = { .016f, 1.f, .016f, .016f, .016f };
    const int replay[] = { 0, 0, 1, 1, 1 };
    const int expected[] = { 0, 3, 0, 3, 1 };
    bool ok = true;
    for (int i = 0; i < 5; ++i) {
        if (i == 3) cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, elapsed[i]);
        const int done = fpas_cui_typewriter(ui, "type", "Stream", replay[i], &options);
        cui_end_frame(ui);
        if (done != expected[i]) {
            fprintf(stderr, "FPAS typewriter frame %d: complete=%d\n", i, done);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool number_roll(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    const char *samples[] = { "1234567890123456789012345", "1 milliseconds per request" };
    bool ok = true;
    for (int i = 0; i < 2; ++i) {
        cui_begin_frame(ui, .016f);
        cui_text(ui, samples[i], .tabular = true);
        cui_end_frame(ui);
        const int expected = cui_get_draw_data(ui)->vertex_count;
        cui_begin_frame(ui, .016f);
        if (adapter) {
            const fpas_motion_text options = { 0 };
            fpas_cui_number_roll(ui, "number", samples[i], &options);
        } else {
            cui_number_roll(ui, "number", samples[i]);
        }
        cui_end_frame(ui);
        const int actual = cui_get_draw_data(ui)->vertex_count;
        if (actual != expected) {
            fprintf(stderr, "%s rolling number %d: glyph geometry has %d vertices, expected %d\n",
                adapter ? "FPAS" : "CUI", i, actual, expected);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool waveform_passive_cancels_seek(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 420, 1);
    const float peaks[] = { .3f, .5f };
    cui_rect bounds = {0};
    bool ok = true;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 1 || frame == 6) {
            cui_input_mouse_move(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 4) {
            cui_input_mouse_move(ui, bounds.x + bounds.w * .8f, bounds.y + bounds.h + 10);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3 || frame == 5 || frame == 7)
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        const bool passive = frame >= 2 && frame <= 4;
        float position = .1f;
        cui_begin_frame(ui, .016f);
        const int changed = fpas_cui_waveform(ui, "wave", peaks, 2, passive ? NULL : &position,
            2, 300, 40, 0, 0, 0, 0, 5);
        cui_end_frame(ui);
        bounds = cui_get_rect(ui, cui_make_id(ui, "wave"));
        const bool committed = frame == 7;
        if (changed != committed || position != (committed ? .5f : .1f)) {
            fprintf(stderr, "Waveform frame %d: position=%g, changed=%d\n", frame, position, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool frame_delta(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_begin_frame(ui, .02f);
    bool ok = cui_delta_time(ui) == .02f;
    cui_end_frame(ui);
    const double first = cui_time(ui);
    cui_begin_frame(ui, .5f);
    ok = ok && cui_delta_time(ui) == .25f && fabs(cui_time(ui) - first - .5) < 1e-6;
    cui_end_frame(ui);
    cui_destroy(ui);
    return ok;
}

static bool waveform_focus(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    const float peaks[] = { .6f, .6f };
    const cui_id id = cui_make_id(ui, "focus-wave");
    int baseline = 0, total = 0;
    float progress = .25f;
    bool ok = true;
    for (int frame = 0; frame < 3; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_TAB);
        if (frame == 2) key(ui, CUI_KEY_RIGHT);
        cui_begin_frame(ui, .016f);
        int changed = fpas_cui_waveform(ui, "focus-wave", peaks, 2, &progress,
            2, 300, 34, 0, 0, 0, 0, 5);
        cui_end_frame(ui);
        const cui_rect bounds = cui_get_rect(ui, id);
        const cui_draw_data *draw = cui_get_draw_data(ui);
        int interior = 0;
        for (int i = 0; i < draw->vertex_count; ++i) {
            const cui_vertex *vertex = &draw->vertices[i];
            if (vertex->x > bounds.x + 8 && vertex->x < bounds.x + bounds.w - 8 &&
                vertex->y > bounds.y + 5 && vertex->y < bounds.y + bounds.h - 5) ++interior;
        }
        if (frame == 0) { baseline = interior; total = draw->vertex_count; }
        if (baseline < 50 || interior != baseline || (frame && draw->vertex_count <= total)
            || changed != (frame == 2) || cui_focused_id(ui) != (frame ? id : 0)) {
            fprintf(stderr, "Waveform focus frame %d: interior=%d, total=%d, changed=%d\n",
                frame, interior, draw->vertex_count, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok && fabsf(progress - .30f) < .001f;
}

static bool audio_release_mode(int mode) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 420, 1);
    const bool live = mode == 0, player = mode == 2;
    const cui_id id = cui_make_id(ui, "audio") ^ (player ? 0x5C8Bu : 0);
    const float peaks[] = { .4f, .6f }, locations[] = { 0, .25f, .8f, .6f, .35f, 0 };
    bool ok = true;
    for (int frame = 0; frame < 6; ++frame) {
        const cui_rect bounds = cui_get_rect(ui, id);
        if (frame >= 1 && frame <= 4) {
            cui_input_mouse_move(ui, bounds.x + bounds.w * locations[frame], bounds.y + bounds.h / 2);
            if (frame != 2) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
            if (frame == 2 || frame == 3) {
                cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
                cui_input_mouse_move(ui, bounds.x + bounds.w + 10, bounds.y + bounds.h + 10);
            }
        }
        if (frame == 5) cui_input_focus(ui, false);
        cui_begin_frame(ui, .016f);
        float position = player ? 10 : .1f;
        int changed;
        if (player) {
            int playing = 0, parts[4];
            changed = fpas_cui_now_playing(ui, "audio", "Track", "", NULL, 0, 0,
                &position, 100, &playing, 2, 360, 2, -1, parts);
            position /= 100;
        } else {
            changed = fpas_cui_waveform(ui, "audio", peaks, 2, &position,
                2, 300, 40, 0, 0, 0, 0, live ? 1 : 5);
        }
        cui_end_frame(ui);
        const bool expected_change = frame == 2 || frame == 3 || (live && (frame == 1 || frame == 4));
        const float expected_position = expected_change ? locations[frame] : .1f;
        if (fabsf(position - expected_position) >= .001f || changed != (expected_change ? (player ? CUI_PLAYER_SEEK : 1) : 0)) {
            fprintf(stderr, "Audio mode %d frame %d: position=%g, changed=%d\n", mode, frame, position, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool audio_release(void) {
    const bool live = audio_release_mode(0);
    const bool deferred = audio_release_mode(1);
    const bool player = audio_release_mode(2);
    return live && deferred && player;
}

/** Draws the rotary control and a second focus target. */
static float knob_input_frame(cui_ctx *ui, bool adapter, float value, float step) {
    int node;
    cui_begin_frame(ui, .07f);
    if (adapter) fpas_cui_knob(ui, "knob", "Knob", &value, 0, 1, step, 0, NULL, NULL,
        0, 48, 0, 0, 0, 0, 1, 0, &node);
    else cui_knob(ui, "knob", &value, 0, 1, .label = "Knob", .step = step);
    cui_button(ui, "Next");
    cui_end_frame(ui);
    return value;
}

/** Checks recorded release positions, Step endpoints, and focused wheel routing. */
static bool knob_input_contract(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 320, 240, 1);
    cui_set_reduced_motion(ui, true);
    float value = knob_input_frame(ui, adapter, .25f, 0);
    const cui_rect bounds = cui_get_rect(ui, cui_make_id(ui, "knob") ^ 0x4B0Bu);
    const float x = bounds.x + bounds.w * .5f, y = bounds.y + bounds.h * .5f;
    cui_input_mouse_move(ui, x, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    value = knob_input_frame(ui, adapter, value, 0);
    cui_input_mouse_move(ui, x, y - 20);
    value = knob_input_frame(ui, adapter, value, 0);
    bool ok = fabsf(value - .35f) < 1e-5f;
    cui_input_mouse_move(ui, x, y - 60);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    cui_input_mouse_move(ui, x, y + 10);
    value = knob_input_frame(ui, adapter, value, 0);
    ok = ok && fabsf(value - (adapter ? .55f : .35f)) < 1e-5f;

    key(ui, CUI_KEY_END);
    value = knob_input_frame(ui, adapter, value, .3f);
    ok = ok && fabsf(value - (adapter ? 1.0f : .9f)) < 1e-5f;
    cui_input_mouse_move(ui, x, y);
    value = knob_input_frame(ui, adapter, value, .3f);
    ok = ok && fabsf(value - (adapter ? 1.0f : .9f)) < 1e-5f;

    key(ui, CUI_KEY_TAB);
    cui_input_scroll(ui, 0, -5);
    value = knob_input_frame(ui, adapter, value, 0);
    ok = ok && fabsf(value - (adapter ? 1.0f : .85f)) < 1e-5f;
    key(ui, CUI_KEY_TAB);
    value = knob_input_frame(ui, adapter, value, 0);
    cui_input_scroll(ui, 0, -5);
    value = knob_input_frame(ui, adapter, value, 0);
    ok = ok && fabsf(value - (adapter ? .95f : .8f)) < 1e-5f;

    if (adapter) {
        key(ui, CUI_KEY_RIGHT);
        value = knob_input_frame(ui, true, value, .3f);
        ok = ok && fabsf(value - .9f) < 1e-5f;
        key(ui, CUI_KEY_HOME);
        value = knob_input_frame(ui, true, value, .3f);
        ok = ok && value == 0;
        cui_input_mouse_move(ui, x, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        cui_input_mouse_move(ui, x, y - 80);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        value = knob_input_frame(ui, true, value, 0);
        ok = ok && fabsf(value - .4f) < 1e-5f;
    }
    if (!ok) fprintf(stderr, "%s knob release, stepping, or focus routing differs.\n", adapter ? "FPAS" : "CUI");
    cui_destroy(ui);
    return ok;
}

static bool motion_vertices_match(cui_ctx *first, cui_ctx *second);

/** Checks explicit knob layout and paint against the same CUI options. */
static bool knob_options_match(void) {
    cui_ctx *raw = cui_create(NULL), *adapter = cui_create(NULL);
    cui_set_viewport(raw, 320, 240, 1);
    cui_set_viewport(adapter, 320, 240, 1);
    cui_set_reduced_motion(raw, true);
    cui_set_reduced_motion(adapter, true);
    bool ok = true;
    const char *format = "~%+08.2E";
    for (int i = 0; i < 3; ++i) {
        const bool log_scale = i == 0, bipolar = i != 0, hide_value = i == 2, enabled = i != 2;
        const float minimum = log_scale ? 20 : -1, maximum = log_scale ? 20000 : 1;
        float a = log_scale ? 1800 : .35f, b = a;
        int node;
        cui_begin_frame(raw, .04f);
        cui_knob(raw, "knob", &a, minimum, maximum, .label = "Cutoff", .format = format, .unit = "Hz",
            .size = 58, .bipolar = bipolar, .log_scale = log_scale, .hide_value = hide_value,
            .disabled = !enabled, .color = CUI_TOKEN_COLOR(CUI_TOKEN_SUCCESS));
        cui_end_frame(raw);
        cui_begin_frame(adapter, .04f);
        const int changed = fpas_cui_knob(adapter, "knob", "Cutoff", &b, minimum, maximum, 0, 0, format, "Hz",
            hide_value, 58, bipolar, log_scale, -CUI_TOKEN_SUCCESS - 1, 0, enabled, 0, &node);
        cui_end_frame(adapter);
        if (changed || !motion_vertices_match(raw, adapter)) ok = false;
    }
    if (!ok) fprintf(stderr, "Knob options changed the native draw geometry or color.\n");
    cui_destroy(adapter);
    cui_destroy(raw);
    return ok;
}

static bool dial_reset(bool knob) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 420, 1);
    bool ok = true;
    const cui_key inputs[] = { CUI_KEY_NONE, CUI_KEY_DELETE, CUI_KEY_NONE, CUI_KEY_DELETE, CUI_KEY_BACKSPACE, CUI_KEY_RIGHT };
    const float expected[] = { .24f, .24f, .24f, 0, 0, .5f };
    cui_rect bounds = {0};
    for (int frame = 0; frame < 9; ++frame) {
        if (frame < 6 && inputs[frame] != CUI_KEY_NONE) key(ui, inputs[frame]);
        if (frame == 6 || frame == 8) {
            cui_input_mouse_move(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 7) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        float value = .24f;
        int node;
        const int changed = knob
            ? fpas_cui_knob(ui, "dial", "Dial", &value, 0, 1, .5f, 0, "%.2f", NULL,
                0, 48, 0, 0, 0, 0, 1, frame == 2, &node)
            : fpas_cui_fader(ui, "dial", "Dial", &value, 0, 1, .5f, 0, "%.2f", NULL,
                0, 160, 0, NULL, 0, NULL, 0, 1, frame == 2, &node);
        const cui_id id = fpas_cui_node_key(ui, node);
        cui_end_frame(ui);
        bounds = cui_get_rect(ui, id);
        const bool key_ok = frame >= 6 || (changed == (frame >= 3) && value == expected[frame]);
        const bool double_click_ok = frame != 8 || (changed && value == 0);
        if (!key_ok || !double_click_ok || fpas_cui_focused(ui, node) != (frame >= 2)) {
            fprintf(stderr, "%s frame %d: value=%g, changed=%d\n", knob ? "Knob" : "Fader", frame, value, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool fader_release_and_bound(bool horizontal) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    float value = .2f;
    cui_rect bounds = {0};
    bool ok = true;
    for (int frame = 0; frame < 10; ++frame) {
        if ((frame >= 2 && frame <= 4) || frame == 8) {
            const float target = frame == 2 ? .2f : frame == 3 ? .6f : frame == 4 ? .8f : .4f;
            const float x = bounds.x + (horizontal ? 8 + (bounds.w - 16) * target : bounds.w / 2);
            const float y = bounds.y + (horizontal ? bounds.h / 2 : bounds.h - 8 - (bounds.h - 16) * target);
            cui_input_mouse_move(ui, x, y);
        }
        if (frame == 2 || frame == 8) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 4 || frame == 8) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 6) key(ui, CUI_KEY_END);
        if (frame == 9) { key(ui, CUI_KEY_TAB); cui_input_scroll(ui, 0, 5); }
        cui_begin_frame(ui, .04f);
        int node;
        const int changed = fpas_cui_fader(ui, "volume", "", &value, 0, 1, frame >= 5 && frame <= 7 ? .3f : 0,
            0, "%.1f", NULL, 1, 160, horizontal, NULL, 0, NULL, 0, 1, 0, &node);
        cui_button(ui, "Next");
        const cui_id next = cui_last_id(ui);
        cui_end_frame(ui);
        bounds = fpas_cui_node_rect(ui, node);
        const float expected = frame < 3 ? .2f : frame == 3 ? .6f : frame < 6 ? .8f : frame < 8 ? 1 : .4f;
        if (fabsf(value - expected) > 1e-5f || changed != (frame == 3 || frame == 4 || frame == 6 || frame == 8) ||
            (frame == 9 && cui_focused_id(ui) != next)) {
            fprintf(stderr, "%s fader frame %d: value=%g, changed=%d\n",
                horizontal ? "Horizontal" : "Vertical", frame, value, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

/** Compares rendered geometry and colors for identical native frames. */
static bool motion_vertices_match(cui_ctx *first, cui_ctx *second) {
    const cui_draw_data *a = cui_get_draw_data(first), *b = cui_get_draw_data(second);
    return a->vertex_count > 0 && a->vertex_count == b->vertex_count &&
        memcmp(a->vertices, b->vertices, (size_t)a->vertex_count * sizeof *a->vertices) == 0;
}

static bool fader_options_match(void) {
    cui_ctx *raw = cui_create(NULL), *adapter = cui_create(NULL);
    const float ticks[] = {-48, -12, 0, 6}, peaks[] = {.2f, .8f};
    cui_set_viewport(raw, 640, 420, 1); cui_set_viewport(adapter, 640, 420, 1);
    cui_set_reduced_motion(raw, true); cui_set_reduced_motion(adapter, true);
    bool ok = true;
    for (int frame = 0; frame < 4; ++frame) {
        const bool horizontal = (frame & 1) != 0, enabled = frame < 2;
        float native_value = -12, value = native_value;
        int node;
        cui_begin_frame(raw, .04f); cui_begin_frame(adapter, .04f);
        cui_fader(raw, "volume", &native_value, -48, 6, .label="Mix", .format="%+.2f", .unit="dB",
            .length=165, .horizontal=horizontal, .step=.5f, .reset=-24,
            .ticks=ticks, .tick_count=4, .meter=peaks, .meter_channels=2, .disabled=!enabled);
        const int changed = fpas_cui_fader(adapter, "volume", "Mix", &value, -48, 6, .5f, -24, "%+.2f", "dB",
            0, 165, horizontal, ticks, 4, peaks, 2, enabled, 0, &node);
        cui_end_frame(raw); cui_end_frame(adapter);
        ok &= !changed && native_value == value && motion_vertices_match(raw, adapter);
    }
    cui_destroy(raw); cui_destroy(adapter);
    if (!ok) fprintf(stderr, "Fader native presentation differs with custom options\n");
    return ok;
}

/** Draws the text effects with custom fonts, colors, widths and timing. */
static void draw_text_options(cui_ctx *ui, bool adapter, float elapsed) {
    const char *text = "A sample sentence that reaches the fixed width.";
    const char *words[] = { "local", "production", "everywhere" };
    const fpas_motion_text bridge = { .font = CUI_FONT_MONO, .width_kind = 2, .width = 115,
        .size = 16, .palette = -1, .rgba = 0xffc67a3au, .speed = 10, .delay = .1f, .flags = 1 };
    const cui_text_fx_opts raw = { .font = CUI_FONT_MONO, .width = CUI_PX(115), .size = 16,
        .color = { 58, 122, 198, 255 }, .speed = 10, .delay = .1f, .nowrap = true };
    cui_begin_frame(ui, elapsed);
    if (adapter) {
        fpas_cui_shimmer_text(ui, "shimmer", text, &bridge);
        fpas_cui_typewriter(ui, "type", text, 0, &bridge);
        fpas_cui_text_reveal(ui, "reveal", text, 0, &bridge);
        fpas_cui_word_rotate(ui, "rotate", words, 3, .7f, &bridge);
    } else {
        cui_shimmer_text_ex(ui, text, &raw);
        cui_typewriter_ex(ui, "type", text, &raw);
        cui_text_reveal_ex(ui, "reveal", text, &raw);
        cui_word_rotate_ex(ui, "rotate", words, 3, .7f, &raw);
    }
    cui_end_frame(ui);
}

/** Checks motion and reduced-motion drawing against the native text effects. */
static bool text_options_match(void) {
    cui_ctx *raw = cui_create(NULL), *bridge = cui_create(NULL);
    cui_set_viewport(raw, 320, 200, 1);
    cui_set_viewport(bridge, 320, 200, 1);
    bool ok = true;
    for (int i = 0; i < 8; ++i) {
        if (i == 6) {
            cui_set_reduced_motion(raw, true);
            cui_set_reduced_motion(bridge, true);
        }
        draw_text_options(raw, false, .2f);
        draw_text_options(bridge, true, .2f);
        if (!motion_vertices_match(raw, bridge) || cui_needs_redraw(raw) != cui_needs_redraw(bridge)) ok = false;
    }
    if (!ok) fprintf(stderr, "Text effect options changed native geometry, colors, or timing.\n");
    cui_destroy(bridge);
    cui_destroy(raw);
    return ok;
}

/** Checks each loader against native motion and a fixed reduced-motion frame. */
static bool loader_motion(void) {
    cui_ctx *raw = cui_create(NULL), *bridge = cui_create(NULL), *still = cui_create(NULL);
    cui_set_viewport(raw, 320, 200, 1);
    cui_set_viewport(bridge, 320, 200, 1);
    cui_set_viewport(still, 320, 200, 1);
    bool ok = true;
    const cui_color color = { 58, 122, 198, 255 };
    for (int i = 0; i < 5; ++i) {
        cui_begin_frame(raw, .13f);
        cui_loader(raw, .style = (cui_loader_style)i, .size = 28, .color = color);
        cui_end_frame(raw);
        cui_set_reduced_motion(bridge, false);
        cui_begin_frame(bridge, .13f);
        fpas_cui_loader(bridge, "loader", i, 28, -1, 0xffc67a3au);
        cui_end_frame(bridge);
        if (!motion_vertices_match(raw, bridge) || !cui_needs_redraw(bridge)) ok = false;

        cui_begin_frame(still, 0);
        cui_loader(still, .style = (cui_loader_style)i, .size = 28, .color = color);
        cui_end_frame(still);
        cui_set_reduced_motion(bridge, true);
        cui_begin_frame(bridge, 0);
        fpas_cui_loader(bridge, "loader", i, 28, -1, 0xffc67a3au);
        cui_end_frame(bridge);
        if (!motion_vertices_match(still, bridge) || cui_needs_redraw(bridge)) ok = false;
    }
    cui_begin_frame(bridge, .16f);
    cui_request_redraw(bridge);
    fpas_cui_loader(bridge, "loader", CUI_LOADER_DOTS, 0, 0, 0);
    cui_end_frame(bridge);
    if (!cui_needs_redraw(bridge)) ok = false;
    if (!ok) fprintf(stderr, "Loader changed native motion or failed reduced motion.\n");
    cui_destroy(still);
    cui_destroy(bridge);
    cui_destroy(raw);
    return ok;
}

/** Counts vertices that contribute visible color in one paint mode. */
static int motion_visible_vertices(cui_ctx *ui, int shade) {
    const cui_draw_data *draw = cui_get_draw_data(ui);
    int count = 0;
    for (int i = 0; i < draw->vertex_count; ++i)
        if (draw->vertices[i].mode == shade && draw->vertices[i].color.a) ++count;
    return count;
}

/** Checks the animated caret, idle wakeup, replay, and reduced motion. */
static bool typewriter_caret(void) {
    cui_ctx *raw = cui_create(NULL), *bridge = cui_create(NULL);
    cui_set_viewport(raw, 320, 200, 1);
    cui_set_viewport(bridge, 320, 200, 1);
    const fpas_motion_text options = { .speed = 2, .delay = 4, .flags = 2 };
    const float deltas[] = { .016f, .55f, .5f, .55f, .016f, .016f };
    const bool expected[] = { true, false, true, false, true, false };
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        cui_begin_frame(raw, deltas[i]);
        cui_typewriter(raw, "message", "Hello", .speed = 2, .delay = 4, .caret = true);
        cui_end_frame(raw);
        if (i == 5) cui_set_reduced_motion(bridge, true);
        cui_begin_frame(bridge, deltas[i]);
        const int state = fpas_cui_typewriter(bridge, "message", "Hello", i >= 4, &options);
        cui_end_frame(bridge);
        const bool caret = motion_visible_vertices(bridge, CUI_SHADE_RRECT) > 0;
        if (motion_visible_vertices(raw, CUI_SHADE_RRECT) == 0 || caret != expected[i] || state != (i == 5 ? 3 : 0))
            ok = false;
        const float wake = cui_next_wakeup(bridge);
        if (i < 5 && !(wake > 0 && wake <= .51f)) ok = false;
    }
    if (!ok) fprintf(stderr, "Typewriter caret timing, wakeup, or completion differed.\n");
    cui_destroy(bridge);
    cui_destroy(raw);
    return ok;
}

/** Compares staggered whitespace to the native text after the reveal completes. */
static bool reveal_whitespace(const char *text) {
    cui_ctx *raw = cui_create(NULL), *bridge = cui_create(NULL);
    cui_set_viewport(raw, 320, 200, 1);
    cui_set_viewport(bridge, 320, 200, 1);
    const fpas_motion_text options = { .speed = 1, .delay = .2f };
    const float elapsed[] = { 0, .85f, 1, 0, 0, .016f };
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        const char *shown = i == 3 ? "A B" : text;
        if (i == 5) {
            cui_set_reduced_motion(raw, true);
            cui_set_reduced_motion(bridge, true);
        }
        cui_begin_frame(raw, elapsed[i]);
        cui_text_reveal(raw, "words", shown, .speed = 1, .delay = .2f);
        cui_end_frame(raw);
        cui_begin_frame(bridge, elapsed[i]);
        fpas_cui_text_reveal(bridge, "words", shown, 0, &options);
        cui_end_frame(bridge);
        const int visible = motion_visible_vertices(bridge, CUI_SHADE_TEXT);
        if (i == 1 && (visible == 0 || visible >= motion_visible_vertices(raw, CUI_SHADE_TEXT))) ok = false;
        if ((i == 2 || i == 5) && !motion_vertices_match(raw, bridge)) ok = false;
        if ((i == 0 || i == 3 || i == 4) && visible != 0) ok = false;
    }
    if (!ok) fprintf(stderr, "Text reveal did not stagger or retain the original whitespace layout.\n");
    cui_destroy(bridge);
    cui_destroy(raw);
    return ok;
}

/** Runs the native motion and adapter regressions. */
int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool navigation = tabs(adapter);
    const bool replay = !adapter || typewriter();
    const bool full_number = number_roll(adapter);
    const bool seek = !adapter || waveform_passive_cancels_seek();
    const bool knob = knob_input_contract(adapter) && (!adapter || (dial_reset(true) && knob_options_match()));
    const bool fader = !adapter || (dial_reset(false) && fader_release_and_bound(true) &&
        fader_release_and_bound(false) && fader_options_match());
    const bool loaders = !adapter || loader_motion();
    const bool caret = !adapter || typewriter_caret();
    const bool whitespace = !adapter || (reveal_whitespace("A\nB") && reveal_whitespace("A\tB"));
    const bool text_options = !adapter || text_options_match();
    const bool cards = !adapter || motion_cards_contract();
    const bool tab_configuration = !adapter || (tab_presentation() && tab_resize());
    const bool dock_configuration = !adapter || dock_presentation();
    const bool retired = !adapter || navigation_retirement();
    const bool release = !adapter || audio_release();
    const bool wave_focus = !adapter || waveform_focus();
    const bool delta = frame_delta();
    return navigation && replay && full_number && seek && knob && fader && loaders && caret && whitespace &&
        text_options && cards && tab_configuration && dock_configuration && retired && release && wave_focus && delta ? 0 : 1;
}
