// [Documentation](../../../../docs/ui/gui/social-and-media/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

/** Measures the counter's glyph run after the native layout. */
static float counter_width(cui_ctx *ui, const char *format, int font) {
    cui_begin_frame(ui, 0.016f);
    fpas_cui_counter(ui, "counter", 25, format, 32, 0, 0, 0, font);
    cui_end_frame(ui);
    return cui_get_rect(ui, cui_make_id(ui, "counter")).w;
}

/** Checks native counter font and Unicode currency, short ticker coverage and text spacing. */
static bool counter_and_ticker_options(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 280, 1);
    cui_set_reduced_motion(ui, true);
    const float display = counter_width(ui, "%.0f", CUI_FONT_DISPLAY);
    const float mono = counter_width(ui, "%.0f", CUI_FONT_MONO);
    const float currency = counter_width(ui, "€%.0f", CUI_FONT_DISPLAY);
    const float glyph = cui_measure_text(ui, CUI_FONT_DISPLAY, 32, "€", -1).width;
    bool ok = mono != display && fabsf(currency - display - glyph) <= 1;
    const cui_id ticker = cui_make_id(ui, "ticker");
    fpas_ticker_item item = { .label = "Activity" };
    float widths[2];
    for (int pass = 0; pass < 2; ++pass) {
        if (pass) item.value = "";
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, 0.016f);
            fpas_cui_ticker(ui, "ticker", &item, 1, 110, 56);
            cui_end_frame(ui);
            if (pass == 0 && frame == 0) ok &= cui_needs_redraw(ui);
        }
        widths[pass] = cui_get_rect(ui, ticker ^ 0x5E7u).w;
        ok &= cui_get_rect(ui, ticker).h == 56 && !cui_needs_redraw(ui);
    }
    ok &= widths[1] > widths[0];
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Counter glyphs or ticker repetition and spacing differ.\n");
    return ok;
}

/** Builds two native slide scopes and exposes the selected value and visibility parts. */
static int carousel_frame(cui_ctx *ui, float width, int index, float elapsed, float advance,
    bool focus, int *parts) {
    cui_begin_frame(ui, elapsed);
    cui_box(ui, .width = CUI_PX(width)) {
        const int node = fpas_cui_carousel_begin(ui, "carousel", 2, index, 120, 18, advance, 15, focus);
        for (int k = 0; k < 2; ++k) {
            fpas_cui_carousel_slide_begin(ui);
            cui_box(ui, .id = k ? "second" : "first", .width = CUI_FILL, .height = CUI_FILL) {
                if (!k) cui_button(ui, "Slide action");
                else cui_text(ui, "Second slide");
            }
            fpas_cui_carousel_slide_end(ui);
        }
        index = fpas_cui_carousel_end(ui, node, 1, parts);
    }
    cui_end_frame(ui);
    return index;
}

/** Checks native slide reflow, automatic movement and keyboard wrap with hidden navigation. */
static bool carousel_options(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 260, 1);
    cui_set_reduced_motion(ui, true);
    const cui_id first = cui_make_id(ui, "first");
    int parts[3];
    for (int frame = 0; frame < 3; ++frame) carousel_frame(ui, 160, 0, 0.016f, 0, false, parts);
    bool ok = cui_get_rect(ui, first).w == 160 && !cui_needs_redraw(ui)
        && parts[1] < 0 && parts[2] < 0;
    carousel_frame(ui, 320, 0, 0.016f, 0, false, parts);
    ok &= cui_needs_redraw(ui);
    carousel_frame(ui, 320, 0, 0.016f, 0, false, parts);
    ok &= cui_get_rect(ui, first).w == 320 && fpas_cui_node_rect(ui, parts[0]).h == 120;
    const cui_rect child = cui_get_rect(ui, cui_make_id(ui, "Slide action"));
    cui_input_mouse_move(ui, child.x + child.w / 2, child.y + child.h / 2);
    ok &= carousel_frame(ui, 320, 0, 0.3f, 0.15f, false, parts) == 0;
    cui_input_mouse_leave(ui);
    ok &= carousel_frame(ui, 320, 0, 0.016f, 0.15f, false, parts) == 1;
    carousel_frame(ui, 320, 0, 0.016f, 0, true, parts);
    cui_input_key(ui, CUI_KEY_LEFT, true, 0);
    cui_input_key(ui, CUI_KEY_LEFT, false, 0);
    ok &= carousel_frame(ui, 320, 0, 0.016f, 0, false, parts) == 1;
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Carousel width, pointer pause or wrapped keyboard selection mismatch.\n");
    return ok;
}

static void social_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

typedef struct { int offset, line, checked; const char *link, *hovered; int hover_changed; } chat_activity;

static chat_activity chat_frame(cui_ctx *ui, const char *body, int options) {
    chat_activity activity;
    cui_begin_frame(ui, .016f);
    activity.link = fpas_cui_chat_message(ui, "message", body, NULL, NULL, NULL, NULL,
        options, 1, &activity.offset, &activity.line, &activity.checked,
        &activity.hovered, &activity.hover_changed);
    cui_end_frame(ui);
    return activity;
}

static bool chat_markdown_activity(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    const char *body = "[Manual](https://example.com/docs)\n\n- [ ] Ready";
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    chat_frame(ui, body, 8);
    social_key(ui, CUI_KEY_TAB);
    const chat_activity passive = chat_frame(ui, body, 8);
    bool valid = passive.offset == -1 && passive.link == NULL && !cui_focused_id(ui);

    chat_frame(ui, body, 24);
    social_key(ui, CUI_KEY_TAB);
    chat_frame(ui, body, 24);
    const cui_rect link = cui_get_rect(ui, cui_focused_id(ui));
    valid = link.w > 0 && link.h > 0 && valid;
    cui_input_mouse_move(ui, link.x + link.w / 2, link.y + link.h / 2);
    const chat_activity entered = chat_frame(ui, body, 24);
    valid = entered.hover_changed && entered.hovered && strcmp(entered.hovered, "https://example.com/docs") == 0 && valid;
    const chat_activity stayed = chat_frame(ui, body, 24);
    valid = !stayed.hover_changed && valid;
    cui_input_mouse_move(ui, 470, 230);
    const chat_activity left = chat_frame(ui, body, 24);
    valid = left.hover_changed && !left.hovered && valid;
    social_key(ui, CUI_KEY_ENTER);
    const chat_activity activated = chat_frame(ui, body, 24);
    valid = activated.link && strcmp(activated.link, "https://example.com/docs") == 0 && valid;

    social_key(ui, CUI_KEY_TAB);
    chat_frame(ui, body, 24);
    social_key(ui, CUI_KEY_ENTER);
    const chat_activity task = chat_frame(ui, body, 24);
    valid = task.offset == strstr(body, "[ ]") - body + 1 && task.line == 3 && task.checked && valid;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Chat Markdown lost native link, hover or task input.\n");
    return valid;
}

static bool comment_like_mode(int initial_count, int initial_liked, int expected_count, int expected_liked) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    int count = initial_count, liked = initial_liked, parts[2], result = 0;
    for (int frame = 0; frame < 4; ++frame) {
        if (frame == 1 || frame == 3) social_key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        result = fpas_cui_comment(ui, "comment", "Maya", "Approved.", "", 1, "",
            3, frame == 0 ? 0 : frame == 2 ? 1 : -1, &count, &liked, parts);
        cui_end_frame(ui);
        if (parts[0] < 0 || parts[1] < 0 || (frame == 1 && result != CUI_COMMENT_LIKE)) {
            cui_destroy(ui);
            return false;
        }
    }
    cui_destroy(ui);
    return count == expected_count && liked == expected_liked && result == CUI_COMMENT_REPLY;
}

static bool comment_uses_optional_state(void) {
    const bool valid = comment_like_mode(-1, -1, -1, -1) && comment_like_mode(3, -1, 3, -1) &&
        comment_like_mode(-1, 0, -1, 1) && comment_like_mode(3, 1, 2, 0);
    if (!valid) fprintf(stderr, "Comment like or reply mode changed its native proposal.\n");
    return valid;
}

static cui_rect presence_frame(cui_ctx *ui, int limit, float size) {
    static const fpas_presence_user users[] = {
        { .name = "Maya", .idle = 1 }, { .name = "Tomas" }, { .name = "Jonas" },
        { .name = "Priya" }, { .name = "Rosa" }, { .name = "Sam" }
    };
    cui_begin_frame(ui, .05f);
    fpas_cui_presence(ui, "here", users, 6, limit, size);
    cui_end_frame(ui);
    return cui_get_rect(ui, cui_make_id(ui, "here"));
}

static bool presence_spreads_with_tooltip(void) {
    enum { WIDTH = 480, HEIGHT = 240 };
    static unsigned char before[WIDTH * HEIGHT * 4], after[WIDTH * HEIGHT * 4];
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    const cui_rect defaults = presence_frame(ui, 0, 0);
    const cui_rect base = presence_frame(ui, 5, 30);
    cui_input_mouse_move(ui, base.x + 12, base.y + base.h / 2);
    const cui_rect spread = presence_frame(ui, 0, 0);
    cui_sw_render(renderer, cui_get_draw_data(ui), before, WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    for (int frame = 0; frame < 16; ++frame) presence_frame(ui, 0, 0);
    cui_sw_render(renderer, cui_get_draw_data(ui), after, WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    const bool tooltip = memcmp(before, after, sizeof before) != 0;
    cui_input_mouse_move(ui, WIDTH - 1, HEIGHT - 1);
    const cui_rect retracted = presence_frame(ui, 0, 0);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    const bool valid = defaults.w == base.w && defaults.h == base.h && spread.w > base.w &&
        retracted.w == base.w && tooltip;
    if (!valid) fprintf(stderr, "Presence lost hover spread, tooltip or native defaults.\n");
    return valid;
}

static int draw_stack(cui_ctx *ui, int count, int enabled, bool defaults, cui_rect *bounds) {
    cui_begin_frame(ui, .016f);
    const int root = fpas_cui_card_stack_begin(ui, "deck", count,
        defaults ? 0 : 2, defaults ? 0 : 240, defaults ? 0 : 120, defaults ? 0 : 2, enabled, 0);
    for (int depth = 0; depth < count; ++depth) {
        fpas_cui_card_stack_item_begin(ui, depth);
        fpas_cui_card_stack_item_end(ui);
    }
    int top;
    const int result = fpas_cui_card_stack_end(ui, root, enabled, &top);
    cui_button(ui, "Next control");
    cui_end_frame(ui);
    if (bounds) *bounds = fpas_cui_node_rect(ui, root);
    return result;
}

static void stack_press_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static bool card_stack_gestures(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 600, 420, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect bounds;
    draw_stack(ui, 0, 1, true, &bounds);
    stack_press_key(ui, CUI_KEY_TAB);
    bool valid = draw_stack(ui, 0, 1, true, NULL) == 0 &&
        cui_focused_id(ui) == cui_make_id(ui, "Next control") && bounds.w == 320 && bounds.h == 344;

    draw_stack(ui, 3, 1, false, &bounds);
    valid &= bounds.w == 240 && bounds.h == 132;
    const float x = bounds.x + 100, y = bounds.y + 45;
    cui_input_mouse_move(ui, x, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_stack(ui, 3, 1, false, NULL);
    cui_input_mouse_move(ui, x + 126, y);
    draw_stack(ui, 3, 1, false, NULL);
    stack_press_key(ui, CUI_KEY_RIGHT);
    valid &= draw_stack(ui, 3, 1, false, NULL) == 0;
    draw_stack(ui, 3, 1, false, NULL);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    stack_press_key(ui, CUI_KEY_LEFT);
    valid &= draw_stack(ui, 3, 1, false, NULL) == 1;
    stack_press_key(ui, CUI_KEY_LEFT);
    valid &= draw_stack(ui, 3, 1, false, NULL) == -1;
    stack_press_key(ui, CUI_KEY_RIGHT);
    valid &= draw_stack(ui, 3, 0, false, NULL) == 0;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Card stack lost its geometry, input gating or gesture ownership.\n");
    return valid;
}

static void card_stack_focus_frame(cui_ctx *ui, bool adapter) {
    if (adapter) {
        draw_stack(ui, 1, 1, false, NULL);
        return;
    }
    cui_begin_frame(ui, .016f);
    const cui_card_stack_opts settings = { .width = CUI_PX(240), .height = 120, .visible = 2 };
    cui_card_stack_begin(ui, "deck", 1, &settings);
    cui_card_stack_item(ui, 0) { }
    cui_card_stack_end(ui);
    cui_button(ui, "Next control");
    cui_end_frame(ui);
}

/** Checks for a keyboard-only focus ring and preserves CUI's pointer appearance. */
static bool card_stack_focus_ring(void) {
    enum { WIDTH = 400, HEIGHT = 240 };
    static unsigned char keyboard[2][WIDTH * HEIGHT * 4], pointer[2][WIDTH * HEIGHT * 4];
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        card_stack_focus_frame(ui, pass != 0);
        stack_press_key(ui, CUI_KEY_TAB);
        card_stack_focus_frame(ui, pass != 0);
        cui_sw_render(renderer, cui_get_draw_data(ui), keyboard[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        const cui_rect deck = cui_get_rect(ui, cui_make_id(ui, "deck"));
        cui_input_mouse_move(ui, deck.x + 120, deck.y + 60);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        card_stack_focus_frame(ui, pass != 0);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        card_stack_focus_frame(ui, pass != 0);
        cui_sw_render(renderer, cui_get_draw_data(ui), pointer[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool valid = memcmp(pointer[0], pointer[1], sizeof pointer[0]) == 0 &&
        memcmp(keyboard[0], keyboard[1], sizeof keyboard[0]) != 0;
    if (!valid) fprintf(stderr, "Card stack focus indicator failed or changed pointer rendering.\n");
    return valid;
}

static bool typing_stops_when_motion_is_reduced(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    for (int i = 0; i < 12; ++i) {
        cui_begin_frame(ui, 0.05f);
        if (adapter) {
            fpas_cui_typing_indicator(ui, "typing", "Jonas");
            fpas_cui_typing_indicator(ui, "anonymous", NULL);
        } else {
            cui_typing_indicator(ui, "Jonas");
            cui_typing_indicator(ui, NULL);
        }
        cui_end_frame(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    if (!stopped) fprintf(stderr, "Reduced-motion typing kept requesting frames.\n");
    return stopped;
}

static bool status_visibility(void) {
    const int states[] = { CUI_STATUS_IDLE, CUI_STATUS_BUSY, CUI_STATUS_SUCCESS, CUI_STATUS_SUCCESS,
        CUI_STATUS_SUCCESS, CUI_STATUS_ERROR, CUI_STATUS_ERROR };
    const float elapsed[] = { 0.01f, 0.05f, 0.05f, 4.0f, 0.05f, 0.05f, 4.0f };
    const int expected[] = { 0, 1, 1, 0, 0, 1, 1 };
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    bool ok = true;
    for (int i = 0; i < (int)(sizeof states / sizeof states[0]); ++i) {
        cui_begin_frame(ui, elapsed[i]);
        if (i == 4) fpas_cui_forget(ui, cui_make_id(ui, "job"));
        const int visible = fpas_cui_status_spinner(ui, "job", states[i], 18, 0, 0, i >= 5);
        cui_end_frame(ui);
        if (visible != expected[i]) ok = false;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Status spinner visibility did not follow idle, busy and result states.\n");
    return ok;
}

/** Compares styled activity indicators and supporting controls with direct CUI calls. */
static bool native_pixels_match(void) {
    enum { WIDTH = 320, HEIGHT = 180 };
    static const float samples[] = { 3, 5, 4, 8, 6, 9, 8 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.05f);
            cui_column(ui, .width = CUI_FILL, .gap = 8, .pad = 16) {
                if (pass) {
                    fpas_cui_chat_divider(ui, "divider", "Today");
                    const int group = fpas_cui_skeleton_group_begin(ui, "group", 1, 0, 10, 1);
                    fpas_cui_skeleton(ui, "bar", 3, 65, 12);
                    fpas_cui_skeleton(ui, "line", 1, 0, 8);
                    fpas_cui_skeleton_group_end(ui, group);
                    fpas_cui_spinner_styled(ui, "spinner", 40, CUI_SPINNER_ORBIT,
                        CUI_TOKEN_ACCENT, 0, CUI_TOKEN_CONTROL_TRACK, 0);
                    fpas_cui_sparkline(ui, "trend", samples, 7, 0, 0, 0, 0, 0);
                } else {
                    cui_chat_divider(ui, "Today");
                    cui_skeleton_group(ui, .dither = true, .gap = 10) {
                        cui_skeleton(ui, CUI_PCT(65), 12);
                        cui_skeleton(ui, CUI_FILL, 8);
                    }
                    cui_spinner(ui, 40, .style = CUI_SPINNER_ORBIT,
                        .color = CUI_COLOR_ACCENT, .track = CUI_TOKEN_COLOR(CUI_TOKEN_CONTROL_TRACK));
                    cui_sparkline(ui, samples, 7, CUI_FIT, 0, CUI_TRANSPARENT);
                }
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool equal = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!equal) fprintf(stderr, "Skeleton group, dither spinner or sparkline differs from direct CUI.\n");
    return equal;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return typing_stops_when_motion_is_reduced(false) ? 0 : 1;
    return typing_stops_when_motion_is_reduced(true) && status_visibility() && native_pixels_match() &&
        chat_markdown_activity() && comment_uses_optional_state() && presence_spreads_with_tooltip() &&
        card_stack_gestures() && card_stack_focus_ring() && counter_and_ticker_options() &&
        carousel_options() ? 0 : 1;
}
