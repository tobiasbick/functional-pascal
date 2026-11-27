// [Documentation](../../../../docs/ui/gui/creative-controls/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key input) {
    cui_input_key(ui, input, true, 0);
    cui_input_key(ui, input, false, 0);
}

/** Checks stable IDs, removal input and keyboard ownership of a popover. */
static bool status_tags(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 400, 300, 1);
    cui_set_reduced_motion(ui, true);
    const cui_key keys[] = {
        CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_SPACE,
        CUI_KEY_DELETE, CUI_KEY_BACKSPACE, CUI_KEY_NONE,
        CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_DELETE, CUI_KEY_BACKSPACE,
        CUI_KEY_ESCAPE, CUI_KEY_ENTER, CUI_KEY_BACKSPACE, CUI_KEY_NONE, CUI_KEY_NONE
    };
    cui_rect first_bounds = {0};
    bool valid = true;
    for (int frame = 0; frame < (int)(sizeof keys / sizeof *keys); ++frame) {
        if (keys[frame] != CUI_KEY_NONE) key(ui, keys[frame]);
        if (frame == 15) {
            cui_input_mouse_move(ui, first_bounds.x + first_bounds.w / 2, first_bounds.y + first_bounds.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 16) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        cui_button(ui, "Anchor");
        if (frame == 7) cui_popover_open(ui, "Options");
        cui_popover(ui, "Options", .width = 150) { cui_text(ui, "Overlay"); }
        int passive, first, second;
        const int display = fpas_cui_status_tag(ui, "display", NULL, CUI_TAG_IN_PROGRESS, NULL, 0, 0, -1, &passive);
        const int a = fpas_cui_status_tag(ui, "first", "Draft", CUI_TAG_CUSTOM, CUI_ICON_TAG,
            CUI_TONE_TEAL, 7, frame == 1 ? 0 : -1, &first);
        const int b = fpas_cui_status_tag(ui, "second", "Draft", CUI_TAG_CUSTOM, CUI_ICON_TAG,
            CUI_TONE_TEAL, frame < 14 ? 7 : 3, -1, &second);
        const cui_id first_id = cui_make_id(ui, "first"), second_id = cui_make_id(ui, "second");
        const bool covered_keys_available = frame < 8 || frame > 11 || cui_key_pressed(ui, keys[frame]);
        cui_end_frame(ui);

        const bool expected_second = (frame >= 3 && frame <= 6) || frame == 13;
        first_bounds = fpas_cui_node_rect(ui, first);
        if (display || a != (frame == 16) || b != expected_second || !covered_keys_available ||
            fpas_cui_node_key(ui, first) != first_id || fpas_cui_node_key(ui, second) != second_id ||
            (frame == 0 && (fabsf(fpas_cui_node_rect(ui, passive).h - 30) > .01f || fabsf(first_bounds.h - 24) > .01f)) ||
            (frame == 1 && cui_focused_id(ui) != first_id) ||
            (frame >= 2 && frame < 14 && cui_focused_id(ui) != second_id) ||
            (frame == 14 && fpas_cui_focused(ui, second))) {
            fprintf(stderr, "Status tag frame %d: removal %d/%d/%d, focused %u, key available %d\n",
                frame, display, a, b, cui_focused_id(ui), (int)covered_keys_available);
            valid = false;
        }
    }
    cui_destroy(ui);
    return valid;
}

static int gradient(cui_ctx *ui, bool adapter, float *value, bool enabled, int focus) {
    int parts[2];
    uint32_t color;
    if (adapter)
        return fpas_cui_gradient_slider(ui, "gradient", "Gradient", value, 0, 1,
            CUI_GRADIENT_OPACITY, NULL, NULL, 0, 0, 0, .3f,
            NULL, NULL, NULL, NULL, enabled, focus, &color, parts);
    return cui_gradient_slider(ui, "Gradient", value, 0, 1, .step = .3f,
        .track = CUI_GRADIENT_OPACITY, .disabled = !enabled);
}

static bool gradient_keys(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    float value = 0;
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        if (!adapter && i == 1) key(ui, CUI_KEY_TAB);
        if (i == 2) key(ui, CUI_KEY_END);
        if (i == 4) key(ui, CUI_KEY_LEFT);
        if (i == 5) value = .85f;
        cui_begin_frame(ui, .016f);
        const int flags = gradient(ui, adapter, &value, i < 5, i == 1 ? 0 : -1);
        cui_end_frame(ui);
        const float expected[] = { 0, 0, 1, 1, .6f, .85f };
        const bool changed = i == 2 || i == 4;
        if (fabsf(value - expected[i]) > .00001f || ((flags & CUI_WELL_CHANGED) != 0) != changed) {
            fprintf(stderr, "%s gradient frame %d: value=%g, flags=%d\n",
                adapter ? "FPAS" : "CUI", i, value, flags);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool ratio_caption_keeps_focus(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    int part;
    float ratio = .5f;
    cui_begin_frame(ui, .016f);
    fpas_cui_ratio_slider(ui, "ratio", "First", "Second", &ratio, CUI_TONE_WARNING, .02f, 40, 2, 400, 1, 0, &part);
    cui_end_frame(ui);
    const cui_id original = fpas_cui_node_key(ui, part);
    key(ui, CUI_KEY_RIGHT);
    cui_begin_frame(ui, .016f);
    const int changed = fpas_cui_ratio_slider(ui, "ratio", "A##B", "Renamed", &ratio,
        CUI_TONE_WARNING, .02f, 40, 2, 400, 1, -1, &part);
    cui_end_frame(ui);
    const cui_rect bounds = fpas_cui_node_rect(ui, part);
    const bool valid = changed && fabsf(ratio - .52f) < .00001f && original != 0 &&
        fpas_cui_node_key(ui, part) == original && fpas_cui_focused(ui, part) &&
        bounds.w == 400 && bounds.h == 40;
    if (!valid) fprintf(stderr, "Ratio caption changed focus or sizing: ratio=%g, width=%g, height=%g, flags=%d.\n",
        ratio, bounds.w, bounds.h, changed);
    cui_destroy(ui);
    return valid;
}

typedef enum { RATIO, GRADIENT, STEP, SLIDER_COUNT } creative_slider;

static int draw_slider(cui_ctx *ui, creative_slider widget, float *value, bool enabled, cui_rect *track) {
    int result, parts[2], stop = (int)*value;
    uint32_t sample;
    cui_begin_frame(ui, .016f);
    switch (widget) {
        case RATIO:
            result = fpas_cui_ratio_slider(ui, "ratio", "First", "Second", value, 0, 0, 0, 1, 0, enabled, -1, parts);
            break;
        case GRADIENT:
            result = fpas_cui_gradient_slider(ui, "gradient", "Hue", value, 0, 1, CUI_GRADIENT_HUE,
                NULL, NULL, 0, 0, 0, 0, NULL, NULL, NULL, NULL, enabled, -1, &sample, parts);
            break;
        default:
            result = fpas_cui_step_slider(ui, "step", "Variations", &stop, 1, 4,
                NULL, 0, "%d variants (100%%)", NULL, NULL, NULL, enabled, -1, parts);
            *value = (float)stop;
            break;
    }
    cui_end_frame(ui);
    if (track) *track = fpas_cui_node_rect(ui, parts[0]);
    return result;
}

static bool slider_release_positions(void) {
    bool valid = true;
    for (creative_slider widget = RATIO; widget < SLIDER_COUNT; ++widget) {
        cui_ctx *ui = cui_create(NULL);
        if (!ui) return false;
        cui_set_viewport(ui, 640, 420, 1);
        cui_set_reduced_motion(ui, true);
        float value = widget == STEP ? 2 : .5f;
        cui_rect track;
        draw_slider(ui, widget, &value, true, &track);
        const float y = track.y + track.h / 2;
        cui_input_mouse_move(ui, track.x + track.w * .30f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        draw_slider(ui, widget, &value, true, NULL);
        cui_input_mouse_move(ui, track.x + track.w * .95f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        const int completed = draw_slider(ui, widget, &value, true, NULL);
        const bool reached_end = widget == STEP ? value == 4 : value > .65f;
        const float end = value;
        cui_input_mouse_move(ui, track.x + track.w * .75f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        cui_input_mouse_move(ui, track.x + track.w * .05f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        const int coalesced = draw_slider(ui, widget, &value, true, NULL);
        const bool reached_start = widget == STEP ? value == 1 : value < .35f;
        const float start = value;
        cui_input_mouse_move(ui, track.x + track.w * .4f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        draw_slider(ui, widget, &value, true, NULL);
        const float previous = value;
        cui_input_mouse_move(ui, track.x + track.w * .82f, y);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        const int disabled = draw_slider(ui, widget, &value, false, NULL);
        if (completed != 1 || !reached_end || coalesced != 1 || !reached_start || disabled != 0 || previous != value) {
            fprintf(stderr, "Slider %d failed its release or disabled check: value=%g/%g/%g, flags=%d/%d/%d.\n",
                widget, end, start, value, completed, coalesced, disabled);
            valid = false;
        }
        cui_destroy(ui);
    }
    return valid;
}

static bool slider_tool_keys(void) {
    bool valid = true;
    for (creative_slider widget = GRADIENT; widget <= STEP; ++widget) {
        cui_ctx *ui = cui_create(NULL);
        if (!ui) return false;
        cui_set_viewport(ui, 640, 420, 1);
        float hue = 120;
        int count = 2, parts[2];
        uint32_t sample;
        for (int frame = 0; frame < 7; ++frame) {
            if (frame == 1 || frame == 3 || frame == 5) key(ui, CUI_KEY_ENTER);
            if (frame == 2 || frame == 4 || frame == 6) key(ui, CUI_KEY_SPACE);
            cui_begin_frame(ui, .016f);
            if (frame == 3) cui_popover_open(ui, "Cover");
            if (frame == 5) cui_popover_close(ui, "Cover");
            if (cui_popover_begin(ui, "Cover", NULL)) {
                cui_text(ui, "Popup");
                cui_popover_end(ui);
            }
            const int flags = widget == GRADIENT
                ? fpas_cui_gradient_slider(ui, "gradient", "Hue", &hue, 0, 360, CUI_GRADIENT_HUE,
                      NULL, NULL, 0, 0, 0, 0, NULL, NULL, CUI_ICON_TARGET, "Pick", 5, frame == 0 ? 1 : -1, &sample, parts)
                : fpas_cui_step_slider(ui, "step", "Variations", &count, 1, 4,
                      NULL, 0, NULL, NULL, CUI_ICON_REFRESH_CW, "Generate", 5, frame == 0 ? 1 : -1, parts);
            cui_end_frame(ui);
            const int expected = (frame == 1 || frame == 2 || frame == 5 || frame == 6) ? CUI_WELL_ACTION : 0;
            if (flags != expected || !fpas_cui_focused(ui, parts[1]) ||
                (widget == GRADIENT && sample != 0xff00ff00u)) {
                fprintf(stderr, "Slider %d tool frame %d: flags=%d, expected=%d.\n", widget, frame, flags, expected);
                valid = false;
            }
        }
        cui_destroy(ui);
    }
    return valid;
}

static int option_caption_vertices(const char *caption) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return -1;
    cui_set_viewport(ui, 640, 420, 1);
    const char *items[] = {caption, "Other"};
    int parts[3], selected = 0;
    cui_begin_frame(ui, .016f);
    fpas_cui_option_action(ui, "options", "Options", items, 2, &selected, NULL, NULL, NULL, 1, -1, parts);
    cui_end_frame(ui);
    const int vertices = cui_get_draw_data(ui)->vertex_count;
    cui_destroy(ui);
    return vertices;
}

static bool literal_option_caption(void) {
    const int expected = option_caption_vertices("ABliteral");
    const int actual = option_caption_vertices("##literal");
    if (actual > 0 && actual == expected) return true;
    fprintf(stderr, "Option action literal caption drew %d vertices, expected %d.\n", actual, expected);
    return false;
}

typedef struct {
    uint32_t colors[2];
    int locked[2];
    uint32_t seed;
    int parts[3];
} palette_sample;

static const cui_color palette_focus_ink = {233, 21, 150, 255};

/** Finds the focus ring centered on a rendered swatch. */
static bool palette_ring_at(cui_ctx *ui, int node) {
    const cui_rect bounds = fpas_cui_node_rect(ui, node);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_RRECT && memcmp(&v->border_color, &palette_focus_ink, sizeof palette_focus_ink) == 0 &&
            fabsf(v->x - v->lx - (bounds.x + bounds.w / 2)) < 1 &&
            fabsf(v->y - v->ly - (bounds.y + bounds.h / 2)) < 1)
            return true;
    }
    return false;
}

/** Draws the palette with the same options on either side of the bridge. */
static int draw_palette(cui_ctx *ui, bool adapter, palette_sample *p, bool seeded, int options) {
    uint32_t *seed = seeded ? &p->seed : NULL;
    if (adapter) return fpas_cui_palette_strip(ui, "palette", "Palette", p->colors, p->locked, 2,
        seed, "Two swatches", 48, options, -1, p->parts);
    cui_color colors[2];
    bool locked[2];
    for (int i = 0; i < 2; ++i) {
        const uint32_t c = p->colors[i];
        colors[i] = (cui_color){c & 255u, (c >> 8) & 255u, (c >> 16) & 255u, c >> 24};
        locked[i] = p->locked[i] != 0;
    }
    cui_push_disabled(ui, !(options & 1));
    const int result = cui_palette_strip(ui, "Palette", colors, options & 4 ? NULL : locked, 2,
        .seed = seed, .value = "Two swatches", .height = 48, .no_shuffle = (options & 2) != 0,
        .plain = (options & 8) != 0);
    cui_pop_disabled(ui);
    for (int i = 0; i < 2; ++i) {
        const cui_color c = colors[i];
        p->colors[i] = (uint32_t)c.r | (uint32_t)c.g << 8 | (uint32_t)c.b << 16 | (uint32_t)c.a << 24;
        if (!(options & 4)) p->locked[i] = locked[i];
    }
    return result;
}

/** Compares lock, navigation and both shuffle proposals with and without palette locks. */
static bool palette_options(bool adapter, cui_direction direction) {
    const uint32_t red = 0xff0000ffu, blue = 0xffff0000u;
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .direction = direction });
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    cui_theme_get(ui)->colors[CUI_TOKEN_FOCUS_RING] = palette_focus_ink;
    palette_sample p = { .colors = { red, blue }, .seed = 7 };
    const cui_key next = direction == CUI_DIRECTION_RTL ? CUI_KEY_LEFT : CUI_KEY_RIGHT;
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ENTER, next,
        CUI_KEY_SPACE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_ENTER, CUI_KEY_SPACE,
        CUI_KEY_TAB, CUI_KEY_SPACE, CUI_KEY_ENTER };
    const int expected[] = { 0, 0, CUI_WELL_CHANGED, 0, CUI_WELL_CHANGED, 0,
        CUI_WELL_CHANGED | CUI_WELL_ACTION, CUI_WELL_ACTION, CUI_WELL_CHANGED | CUI_WELL_ACTION, 0, 0, 0, 0 };
    const int bare = 2 | 8, unlocked = 1 | 4;
    const int options[] = { 1, 1, 1, 1, 1, 1, 1, 1, unlocked, bare | unlocked, bare | unlocked, bare | unlocked, bare };
    bool ok = true;
    for (int i = 0; i < (int)(sizeof keys / sizeof *keys); ++i) {
        if (keys[i]) key(ui, keys[i]);
        cui_begin_frame(ui, .016f);
        const int result = draw_palette(ui, adapter, &p, i != 7, options[i]);
        cui_end_frame(ui);
        ok &= result == expected[i];
        if (adapter) {
            ok &= fpas_cui_node_rect(ui, p.parts[0]).h == 48;
            ok &= (p.parts[2] < 0) == ((options[i] & 2) != 0);
            if (i == 9) ok &= cui_focused_id(ui) == 0;
            if (i >= 1 && i <= 4) {
                const int focused = i < 3 ? 0 : 1;
                ok &= fpas_cui_focused(ui, p.parts[focused]) && palette_ring_at(ui, p.parts[focused]);
                ok &= !palette_ring_at(ui, p.parts[1 - focused]);
            }
        }
        if (i == 2) ok &= p.locked[0] && !p.locked[1];
        if (i >= 4) ok &= p.locked[0] && p.locked[1];
        if (i < 8) ok &= p.colors[0] == red && p.colors[1] == blue;
        if (i < 6) ok &= p.seed == 7;
        if (i >= 6) ok &= p.seed != 7;
        if (i >= 8) ok &= p.colors[0] != red && p.colors[1] != blue;
        if (!ok) {
            fprintf(stderr, "%s palette options frame %d: flags=%d seed=%u\n",
                adapter ? "FPAS" : "CUI", i, result, p.seed);
            break;
        }
    }
    cui_destroy(ui);
    return ok;
}

/** Retires a focused swatch when the palette becomes shorter. */
static bool palette_retires_swatch(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    palette_sample p = { .colors = { 0xff0000ffu, 0xffff0000u } };
    bool ok = true;
    for (int i = 0; i < 5; ++i) {
        if (i == 4) key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        const int result = fpas_cui_palette_strip(ui, "palette", "Palette", p.colors, p.locked,
            i == 2 ? 1 : 2, NULL, NULL, 0, 1, i == 1 ? 1 : -1, p.parts);
        cui_end_frame(ui);
        ok &= result == 0 && !p.locked[1] && ((cui_focused_id(ui) != 0) == (i == 1));
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "A swatch retained input after it left the palette.\n");
    return ok;
}

/** Draws the seed field with equivalent state and captions. */
static int draw_seed(cui_ctx *ui, bool adapter, uint32_t *seed, int *locked, int options) {
    const char *display = "Image generation", *random = "Vary", *fixed = "Repeat";
    if (adapter) {
        int parts[2];
        return fpas_cui_seed_field(ui, "seed", "Seed", seed, locked, display, random, fixed, options, -1, parts);
    }
    bool lock = locked && *locked != 0;
    cui_push_disabled(ui, !(options & 1));
    const int result = cui_seed_field(ui, "Seed", seed, locked ? &lock : NULL,
        .value = display, .random_text = random, .fixed_text = fixed, .plain = (options & 8) != 0);
    cui_pop_disabled(ui);
    if (locked) *locked = lock;
    return result;
}

/** Checks that a lock blocks rolls and that the dice works without a lock. */
static bool seed_options(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    uint32_t seed = 1234567890u, rolled = seed;
    int locked = 0;
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_TAB, CUI_KEY_SPACE,
        CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_SPACE, CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_SPACE };
    const int roll = CUI_WELL_ACTION | CUI_WELL_CHANGED;
    const int expected[] = {0, 0, roll, 0, CUI_WELL_CHANGED, 0, 0, CUI_WELL_CHANGED, 0, 0, roll, 0};
    bool ok = true;
    for (int i = 0; i < (int)(sizeof keys / sizeof *keys); ++i) {
        if (keys[i]) key(ui, keys[i]);
        cui_begin_frame(ui, .016f);
        const int result = draw_seed(ui, adapter, &seed, i < 8 ? &locked : NULL, i == 11 ? 8 : 1 | 8);
        cui_end_frame(ui);
        ok &= result == expected[i];
        if (adapter && i == 8) ok &= cui_focused_id(ui) == 0;
        ok &= locked == (i >= 4 && i < 7);
        ok &= result == roll ? seed >= 1000000000u && seed != rolled : seed == rolled;
        if (result == roll) rolled = seed;
        if (!ok) {
            fprintf(stderr, "%s seed options frame %d: flags=%d seed=%u lock=%d\n",
                adapter ? "FPAS" : "CUI", i, result, seed, locked);
            break;
        }
    }
    cui_destroy(ui);
    return ok;
}

/** Leaves covered keyboard input available while preserving the focused part. */
static bool palette_seed_covered(bool palette) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    palette_sample p = { .colors = { 0xff0000ffu, 0xffff0000u }, .seed = 7 };
    uint32_t seed = 1234567890u;
    cui_id focus = 0;
    bool ok = true;
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_RIGHT };
    for (int i = 0; i < 5; ++i) {
        if (keys[i]) key(ui, keys[i]);
        cui_begin_frame(ui, .016f);
        if (i >= 2) {
            cui_button(ui, "Anchor");
            cui_popover_open(ui, "Cover");
            cui_popover(ui, "Cover") { cui_text(ui, "Popup content"); }
        }
        const int result = palette ? draw_palette(ui, true, &p, true, 1) : draw_seed(ui, true, &seed, NULL, 1);
        if (i == 1) focus = cui_focused_id(ui);
        if (i >= 2) ok &= focus && focus == cui_focused_id(ui) && cui_key_pressed(ui, keys[i]);
        ok &= result == 0 && !p.locked[0] && seed == 1234567890u;
        cui_end_frame(ui);
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "The covered %s acted on keys owned by the popover.\n", palette ? "palette" : "seed field");
    return ok;
}

/** Checks that reduced motion stops a rolling field and displays the accepted seed. */
static bool seed_reduced_motion(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    uint32_t seed = 1234567890u;
    bool ok = true;
    int rolling = 0, reduced = 0, settled = 0;
    for (int i = 0; i < 6; ++i) {
        if (i == 1) key(ui, CUI_KEY_TAB);
        if (i == 2) key(ui, CUI_KEY_ENTER);
        if (i == 4) cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, i == 5 ? 1.0f : .016f);
        const int result = draw_seed(ui, true, &seed, NULL, 1);
        ok &= result == (i == 2 ? CUI_WELL_ACTION | CUI_WELL_CHANGED : 0);
        cui_end_frame(ui);
        const int vertices = cui_get_draw_data(ui)->vertex_count;
        if (i == 3) rolling = vertices;
        if (i == 4) reduced = vertices;
        if (i == 5) settled = vertices;
    }
    cui_destroy(ui);
    ok &= rolling > settled && reduced == settled && seed != 1234567890u;
    if (!ok) fprintf(stderr, "Seed field motion vertices: rolling=%d reduced=%d settled=%d.\n", rolling, reduced, settled);
    return ok;
}

/** Runs palette and seed interactions through both the CUI and FPAS calls. */
static bool palette_seed_controls(bool adapter) {
    const bool palette = palette_options(adapter, CUI_DIRECTION_LTR);
    const bool rtl = palette_options(adapter, CUI_DIRECTION_RTL);
    const bool seed = seed_options(adapter);
    if (!adapter) return palette && rtl && seed;
    const bool palette_covered = palette_seed_covered(true);
    const bool seed_covered = palette_seed_covered(false);
    const bool reduced = seed_reduced_motion();
    const bool retired = palette_retires_swatch();
    return palette && rtl && seed && palette_covered && seed_covered && reduced && retired;
}

static bool well_caption_keeps_focus(void) {
    const char *labels[] = {"Note", "C#", "#", "##hidden"};
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 320, 180, 1);
    float value = 0;
    unsigned int track = 0;
    bool valid = true;
    for (int i = 0; i < 4; ++i) {
        uint32_t sample;
        int parts[2];
        cui_begin_frame(ui, .016f);
        fpas_cui_gradient_slider(ui, "###level", labels[i], &value, 0, 1,
            CUI_GRADIENT_OPACITY, NULL, NULL, 0, 0, 0, .1f,
            NULL, NULL, NULL, NULL, 1, i == 0 ? 0 : -1, &sample, parts);
        cui_end_frame(ui);
        if (!i) track = fpas_cui_node_key(ui, parts[0]);
        valid &= track && fpas_cui_node_key(ui, parts[0]) == track && fpas_cui_focused(ui, parts[0]);
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "A well caption changed the slider identity or keyboard focus.\n");
    return valid;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool tags = !adapter || status_tags();
    const bool gradient = gradient_keys(adapter);
    const bool ratio_captions = !adapter || ratio_caption_keeps_focus();
    const bool slider_releases = !adapter || slider_release_positions();
    const bool slider_tools = !adapter || slider_tool_keys();
    const bool well_captions = !adapter || well_caption_keeps_focus();
    const bool palette_seed = palette_seed_controls(adapter);
    const bool captions = !adapter || literal_option_caption();
    return tags && gradient && ratio_captions && slider_releases && slider_tools &&
        well_captions && palette_seed && captions ? 0 : 1;
}
