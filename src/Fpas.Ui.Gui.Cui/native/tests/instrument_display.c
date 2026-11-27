// [Documentation](../../../../docs/ui/gui/instruments/README.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

enum { WIDTH = 400, HEIGHT = 420 };
static unsigned char first[WIDTH * HEIGHT * 4], second[WIDTH * HEIGHT * 4];
static int failures;

static void check(bool condition, const char *message) {
    if (condition) return;
    fprintf(stderr, "%s\n", message);
    ++failures;
}

static cui_ctx *context(cui_sw_renderer *renderer) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer), .theme = CUI_THEME_DARK });
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    return ui;
}

static void render(cui_ctx *ui, cui_sw_renderer *renderer, unsigned char *pixels) {
    cui_sw_render(renderer, cui_get_draw_data(ui), pixels, WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
}

static float lit_position(cui_rect area, const unsigned char *pixels) {
    int sum = 0, count = 0;
    for (int y = (int)area.y; y < (int)(area.y + area.h); ++y) {
        for (int x = (int)area.x; x < (int)(area.x + area.w); ++x) {
            if (pixels[(y * WIDTH + x) * 4] <= pixels[(WIDTH - 1) * 4] + 45) continue;
            sum += x - (int)area.x;
            ++count;
        }
    }
    return count ? sum / (count * area.w) : -1;
}

static void readout_alignment(cui_sw_renderer *renderer, cui_direction direction) {
    cui_ctx *ui = context(renderer);
    const char *ids[] = { "start", "center", "right", "left", "end" };
    cui_begin_frame(ui, .016f);
    cui_column(ui, .direction = direction, .width = CUI_PX(300)) {
        for (int align = CUI_TEXT_START; align <= CUI_TEXT_END; ++align) {
            fpas_cui_readout(ui, ids[align], "LABEL", "54", "MG", "READOUT DETAIL",
                CUI_SIZE_MD, CUI_TONE_NEUTRAL, 0, align, 1, 0);
        }
    }
    cui_end_frame(ui);
    render(ui, renderer, first);
    float positions[5];
    for (int k = 0; k < 5; ++k) {
        const cui_rect area = cui_get_rect(ui, cui_make_id(ui, ids[k]));
        check(area.w == 300 && area.h > 0, "The readout must report its allocated bounds.");
        positions[k] = lit_position(area, first);
    }
    const bool rtl = direction == CUI_DIRECTION_RTL;
    const float start = positions[CUI_TEXT_START], end = positions[CUI_TEXT_END];
    check(positions[CUI_TEXT_LEFT] >= 0 && positions[CUI_TEXT_LEFT] < .25f,
        "Physical left must place the readout at the left edge.");
    check(positions[CUI_TEXT_RIGHT] > .75f, "Physical right must place the readout at the right edge.");
    check(positions[CUI_TEXT_CENTER] > .4f && positions[CUI_TEXT_CENTER] < .6f,
        "Center must place the readout at the center.");
    check(rtl ? start > .75f && end >= 0 && end < .25f : start >= 0 && start < .25f && end > .75f,
        "Logical start and end must follow the layout direction.");
    cui_destroy(ui);
}

static void display_frame(cui_ctx *ui, float width, bool extra_child) {
    cui_begin_frame(ui, .05f);
    cui_column(ui, .width = CUI_PX(width)) {
        if (extra_child) cui_box(ui, .height = CUI_PX(1)) {}
        fpas_cui_dot_display(ui, "marquee", "LONG MSG", 6, 0, 0, CUI_TEXT_START, 1, 0, 4);
    }
    cui_column(ui, .width = CUI_PX(350)) {
        fpas_cui_dot_display(ui, "wide", "OK", 6, 0, 0, CUI_TEXT_START, 1, 0, 0);
    }
    cui_end_frame(ui);
}

static void display_marquee(cui_sw_renderer *renderer) {
    cui_ctx *ui = context(renderer);
    const cui_id marquee = cui_make_id(ui, "marquee"), wide = cui_make_id(ui, "wide");
    for (int frame = 0; frame < 3; ++frame) {
        cui_begin_frame(ui, .016f);
        cui_end_frame(ui);
    }

    display_frame(ui, 90, false);
    check(cui_next_wakeup(ui) > 0 && cui_next_wakeup(ui) < .1f,
        "A newly shown narrow marquee must request a wakeup.");
    const cui_rect original = cui_get_rect(ui, marquee);
    check(original.w == 90 && cui_get_rect(ui, wide).w == 350,
        "Each dot display must report its own bounds.");
    render(ui, renderer, first);

    display_frame(ui, 90, false);
    check(cui_next_wakeup(ui) > 0 && cui_next_wakeup(ui) < .1f,
        "A following wide display must leave the narrow marquee running.");
    render(ui, renderer, second);
    check(memcmp(first, second, sizeof first) != 0, "A narrow marquee must advance the text.");

    display_frame(ui, 350, true);
    const cui_rect moved = cui_get_rect(ui, marquee);
    check(moved.w == 350 && moved.y > original.y, "A reordered dot display must retain its ID.");
    check(cui_next_wakeup(ui) < 0, "A marquee that fits must stop requesting wakeups.");

    display_frame(ui, 90, true);
    check(cui_next_wakeup(ui) > 0 && cui_next_wakeup(ui) < .1f,
        "A newly narrowed marquee must request a wakeup.");
    cui_set_reduced_motion(ui, true);
    display_frame(ui, 90, true);
    check(cui_next_wakeup(ui) < 0, "Reduced motion must stop marquee wakeups.");
    render(ui, renderer, first);
    display_frame(ui, 90, true);
    render(ui, renderer, second);
    check(memcmp(first, second, sizeof first) == 0, "Reduced motion must keep the text still.");
    cui_destroy(ui);
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    readout_alignment(renderer, CUI_DIRECTION_LTR);
    readout_alignment(renderer, CUI_DIRECTION_RTL);
    display_marquee(renderer);
    cui_sw_destroy(renderer);
    return failures ? 1 : 0;
}
