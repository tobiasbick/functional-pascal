/* Compare calendar views and keyboard navigation with direct CUI pixels. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { WIDTH = 400, HEIGHT = 440, CAPACITY = 45, EPOCH = 719162 };
static int failures;
#define CHECK(test) do { if (!(test)) { fprintf(stderr, "line %d: %s\n", __LINE__, #test); ++failures; } } while (0)

typedef struct {
    cui_ctx *ui;
    cui_sw_renderer *sw;
    unsigned char *pixels;
    cui_date date;
    int day, root, parts[CAPACITY], nodes[CAPACITY], count;
} calendar;

static int node_for(calendar *c, int part) {
    for (int i = 0; i < c->count; ++i) if (c->parts[i] == part) return c->nodes[i];
    return -1;
}

static void draw(calendar *c, bool adapter, bool picker, int focus) {
    cui_begin_frame(c->ui, .04f);
    cui_column(c->ui, .pad = 10) {
        if (adapter) {
            c->day = picker
                ? fpas_cui_date_picker(c->ui, "cal", c->day, NULL, NULL, 0, 1, focus,
                    &c->root, c->parts, c->nodes, &c->count)
                : fpas_cui_calendar(c->ui, "cal", c->day, 1, focus,
                    &c->root, c->parts, c->nodes, &c->count);
        } else cui_calendar(c->ui, "cal", &c->date);
    }
    cui_end_frame(c->ui);
    cui_sw_render(c->sw, cui_get_draw_data(c->ui), c->pixels, WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
}

static calendar create(bool dark) {
    calendar c = {0};
    c.sw = cui_sw_create();
    c.ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(c.sw) });
    c.pixels = malloc(WIDTH * HEIGHT * 4);
    c.date = (cui_date){2026, 11, 14};
    c.day = cui_date_to_days(c.date) + EPOCH;
    cui_set_viewport(c.ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(c.ui, true);
    cui_set_theme(c.ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    return c;
}

static void destroy(calendar *c) {
    free(c->pixels);
    cui_destroy(c->ui);
    cui_sw_destroy(c->sw);
}

static void compare(calendar pair[2]) {
    draw(&pair[0], false, false, 0);
    draw(&pair[1], true, false, 0);
    CHECK(memcmp(pair[0].pixels, pair[1].pixels, WIDTH * HEIGHT * 4) == 0);
    CHECK(pair[1].day == cui_date_to_days(pair[0].date) + EPOCH);
}

static void key(calendar pair[2], cui_key code, int count) {
    for (int n = 0; n < count; ++n) {
        for (int pass = 0; pass < 2; ++pass) {
            cui_input_key(pair[pass].ui, code, true, 0);
            cui_input_key(pair[pass].ui, code, false, 0);
        }
        compare(pair);
    }
}

static void check_views(bool dark) {
    calendar pair[] = { create(dark), create(dark) };
    for (int f = 0; f < 3; ++f) compare(pair);
    CHECK(node_for(&pair[1], -8) >= 0 && node_for(&pair[1], -9) >= 0 && node_for(&pair[1], -10) >= 0);
    CHECK(pair[1].count == 45);
    key(pair, CUI_KEY_TAB, 1);
    key(pair, CUI_KEY_ENTER, 1);
    CHECK(pair[1].count == 15);
    CHECK(node_for(&pair[1], -100 - 2026 * 12) >= 0);
    key(pair, CUI_KEY_ENTER, 1);
    CHECK(pair[1].count == 15);
    CHECK(node_for(&pair[1], -200000 - 2016) >= 0);
    key(pair, CUI_KEY_TAB, 1);
    key(pair, CUI_KEY_ENTER, 1);
    CHECK(node_for(&pair[1], -200000 - 2004) >= 0);
    key(pair, CUI_KEY_TAB, 8);
    key(pair, CUI_KEY_ENTER, 1);
    compare(pair); /* The choice frame still contains the previous year's grid. */
    CHECK(node_for(&pair[1], -100 - 2010 * 12) >= 0);
    key(pair, CUI_KEY_TAB, 7);
    key(pair, CUI_KEY_ENTER, 1);
    compare(pair);
    key(pair, CUI_KEY_TAB, 22);
    key(pair, CUI_KEY_ENTER, 1);
    CHECK(pair[1].day == cui_date_to_days((cui_date){2010, 5, 15}) + EPOCH);
    destroy(&pair[0]);
    destroy(&pair[1]);
}

static void click(calendar *c, cui_rect rect, bool picker) {
    cui_input_mouse_move(c->ui, rect.x + rect.w / 2, rect.y + rect.h / 2);
    draw(c, true, picker, 0);
    cui_input_mouse_button(c->ui, CUI_MOUSE_LEFT, true);
    draw(c, true, picker, 0);
    cui_input_mouse_button(c->ui, CUI_MOUSE_LEFT, false);
    draw(c, true, picker, 0);
    draw(c, true, picker, 0);
}

static void check_picker(void) {
    calendar c = create(false);
    draw(&c, true, true, 0);
    draw(&c, true, true, 0);
    click(&c, fpas_cui_node_rect(c.ui, c.root), true);
    for (int f = 0; f < 10; ++f) draw(&c, true, true, 0);
    CHECK(c.count == 45 && node_for(&c, -10) >= 0);
    click(&c, fpas_cui_node_rect(c.ui, node_for(&c, -10)), true);
    CHECK(c.count == 15 && node_for(&c, -100 - 2026 * 12) >= 0);
    cui_input_key(c.ui, CUI_KEY_ESCAPE, true, 0);
    cui_input_key(c.ui, CUI_KEY_ESCAPE, false, 0);
    for (int f = 0; f < 10; ++f) draw(&c, true, true, 0);
    CHECK(c.count == 0);
    click(&c, fpas_cui_node_rect(c.ui, c.root), true);
    for (int f = 0; f < 10; ++f) draw(&c, true, true, 0);
    CHECK(c.count == 45);
    destroy(&c);
}

static void check_requested_focus(void) {
    calendar c = create(false);
    for (int focus = 1; focus <= 3; ++focus) {
        draw(&c, true, false, focus);
        CHECK(fpas_cui_focused(c.ui, node_for(&c, focus == 3 ? -10 : focus == 2 ? -9 : -8)));
    }
    destroy(&c);
}

int main(void) {
    check_views(false);
    check_views(true);
    check_picker();
    check_requested_focus();
    printf("Calendar pixels and navigation: %s (%d failures)\n", failures ? "FAILED" : "passed", failures);
    return failures ? 1 : 0;
}
