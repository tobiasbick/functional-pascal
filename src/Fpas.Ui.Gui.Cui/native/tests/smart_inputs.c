// [Documentation](../../../../docs/ui/gui/smart-inputs/README.md)
/* Native smart inputs parity; decimal money drafts stay text across the ABI. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    cui_ctx *ui;
    cui_sw_renderer *sw;
    unsigned char *pixels;
    char text[128];
    float number;
    char money[128];
    int key, mods, node;
} sample;
static int failures, comparisons;
#define CHECK(test) do { if (!(test)) { fprintf(stderr, "line %d: %s\n", __LINE__, #test); ++failures; } } while (0)

static sample create(bool dark, int width) {
    sample s = {0};
    s.sw = cui_sw_create();
    s.ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(s.sw) });
    s.pixels = malloc((size_t)width * 260 * 4);
    cui_set_viewport(s.ui, width, 260, 1);
    cui_set_reduced_motion(s.ui, true);
    cui_set_theme(s.ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    snprintf(s.text, sizeof s.text, "%s", "Q3 roadmap");
    snprintf(s.money,sizeof s.money,"%s","1,249.50");
    s.number = 12; s.key = CUI_KEY_P; s.mods = CUI_MOD_PRIMARY | CUI_MOD_SHIFT;
    return s;
}

static void destroy(sample *s) {
    free(s->pixels); cui_destroy(s->ui); cui_sw_destroy(s->sw);
}

static void draw(sample *s, bool adapter, int kind, bool fixed, int width) {
    cui_begin_frame(s->ui, .04f);
    cui_column(s->ui, .pad = 16, .gap = 8) {
        unsigned int owner;
        switch (kind) {
        case 0:
            if (adapter) fpas_cui_inline_edit(s->ui, "inline", s->text, sizeof s->text, "Add an owner",
                CUI_FONT_DISPLAY, 26, fixed ? 2 : 0, 200, 1, 0, &s->key, &s->node, &owner);
            else cui_inline_edit(s->ui, "inline", s->text, sizeof s->text,
                .placeholder = "Add an owner", .font = CUI_FONT_DISPLAY, .size = 26,
                .width = fixed ? CUI_PX(200) : CUI_FIT);
            break;
        case 1:
            if (adapter) fpas_cui_scrub_field(s->ui, "###scrub", "Radius", &s->number, 0, 48, .5f,
                "%.1f", "px", NULL, fixed ? 2 : 0, 200, 1, 0, &s->node, &owner);
            else cui_scrub(s->ui, "Radius", &s->number, 0, 48, .step = .5f, .format = "%.1f", .suffix = "px",
                .width = fixed ? CUI_PX(200) : CUI_FIT);
            break;
        case 2:
            if (adapter) fpas_cui_money_field(s->ui, "amount", s->money, sizeof s->money, "Amount", "$", "USD", "Helper", NULL, "0.00",
                fixed ? 2 : 0, 200, 1, 0, &s->node);
            else cui_input(s->ui, "amount", s->money, sizeof s->money, .label = "Amount", .prefix = "$", .suffix = "USD", .hint = "Helper", .placeholder = "0.00",
                .width = fixed ? CUI_PX(200) : CUI_FIT);
            break;
        case 3: {
            int nodes[2];
            cui_key_combo combo = { s->mods, (cui_key)s->key };
            if (adapter) fpas_cui_shortcut_recorder(s->ui, "combo", &s->key, &s->mods, "Record shortcut",
                fixed ? 2 : 0, 200, 1, 0, nodes);
            else { cui_shortcut_recorder(s->ui, "combo", &combo, .placeholder = "Record shortcut",
                .width = fixed ? CUI_PX(200) : CUI_FIT); s->key = combo.key; s->mods = combo.mods; }
            break;
        }
        case 4: {
            char label[128];
            if (adapter) fpas_cui_shortcut_display(s->ui, s->key, s->mods, label, sizeof label);
            else cui_kbd(s->ui, cui_shortcut_label(s->ui, (cui_key_combo){s->mods, (cui_key)s->key}));
            break;
        }
        }
    }
    cui_end_frame(s->ui);
    cui_sw_render(s->sw, cui_get_draw_data(s->ui), s->pixels, width, 260, width * 4, CUI_SW_RGBA);
}

static void compare(sample pair[2], int kind, bool fixed, int width) {
    draw(&pair[0], false, kind, fixed, width); draw(&pair[1], true, kind, fixed, width);
    ++comparisons;
    if (memcmp(pair[0].pixels, pair[1].pixels, (size_t)width * 260 * 4)) {
        fprintf(stderr, "Pixel mismatch: kind=%d fixed=%d width=%d comparison=%d\n", kind, fixed, width, comparisons);
        ++failures;
        const char *dir = getenv("FPAS_SMART_ARTIFACTS");
        if (dir) for (int pass = 0; pass < 2; ++pass) {
            char path[512]; snprintf(path, sizeof path, "%s/widget-%d-%d-%d.png", dir, kind, comparisons, pass);
            cui_write_png(path, pair[pass].pixels, width, 260, width * 4);
        }
    }
}

static void key(sample *s, cui_key code, int mods) {
    cui_input_key(s->ui, code, true, mods); cui_input_key(s->ui, code, false, mods);
}

static void pixels(void) {
    for (int kind = 0; kind < 5; ++kind) for (int dark = 0; dark < 2; ++dark)
        for (int fixed = 0; fixed < 2; ++fixed) for (int narrow = 0; narrow < 2; ++narrow) {
            int width = narrow ? 280 : 640;
            sample pair[] = {create(dark != 0, width), create(dark != 0, width)};
            for (int frame = 0; frame < 3; ++frame) compare(pair, kind, fixed != 0, width);
            if (kind < 4) {
                for (int pass = 0; pass < 2; ++pass) key(&pair[pass], CUI_KEY_TAB, 0);
                compare(pair, kind, fixed != 0, width);
                if (kind == 0 || kind == 1 || kind == 3) {
                    for (int pass = 0; pass < 2; ++pass) key(&pair[pass], CUI_KEY_ENTER, 0);
                    compare(pair, kind, fixed != 0, width); compare(pair, kind, fixed != 0, width);
                    for (int pass = 0; pass < 2; ++pass) key(&pair[pass], CUI_KEY_ESCAPE, 0);
                    compare(pair, kind, fixed != 0, width);
                }
            }
            destroy(&pair[0]); destroy(&pair[1]);
        }
}

static bool inline_frame(sample *s, bool enabled, bool focus) {
    int editing;
    unsigned int key;
    snprintf(s->text, sizeof s->text, "%s", "Original");
    /* Advance past native undo coalescing between consecutive text changes. */
    cui_begin_frame(s->ui, .55f);
    const int flags = fpas_cui_inline_edit(s->ui, "undo", s->text, sizeof s->text, NULL,
        CUI_FONT_MEDIUM, 0, 0, 0, enabled, focus, &editing, &s->node, &key);
    cui_end_frame(s->ui);
    CHECK((flags & CUI_CHANGED) == 0);
    return editing != 0;
}

static void inline_cancel_undo(void) {
    for (int disable = 0; disable <= 1; ++disable) {
        sample s = create(false, 640);
        CHECK(!inline_frame(&s, true, true));
        key(&s, CUI_KEY_ENTER, 0);
        CHECK(inline_frame(&s, true, false));
        cui_input_text(s.ui, "Discard");
        CHECK(inline_frame(&s, true, false));
        cui_input_text(s.ui, " more");
        CHECK(inline_frame(&s, true, false));
        CHECK(strcmp(s.text, "Discard more") == 0);
        key(&s, CUI_KEY_Z, CUI_MOD_PRIMARY);
        CHECK(inline_frame(&s, true, false));
        CHECK(strcmp(s.text, "Discard") == 0);

        cui_input_key(s.ui, CUI_KEY_NONE, false, 0);
        cui_input_text(s.ui, " later");
        CHECK(inline_frame(&s, true, false));
        CHECK(strcmp(s.text, "Discard later") == 0);
        if (!disable) key(&s, CUI_KEY_ESCAPE, 0);
        CHECK(!inline_frame(&s, !disable, false));
        CHECK(strcmp(s.text, "Original") == 0);
        CHECK(!inline_frame(&s, true, true));
        key(&s, CUI_KEY_ENTER, 0);
        CHECK(inline_frame(&s, true, false));
        key(&s, CUI_KEY_Z, CUI_MOD_PRIMARY);
        CHECK(inline_frame(&s, true, false));
        CHECK(strcmp(s.text, "Original") == 0);
        destroy(&s);
    }
}

static void numeric_regressions(void) {
    for (int pass = 0; pass < 2; ++pass) {
        for (int invalid = 0; invalid < 2; ++invalid) {
            sample s = create(false, 640);
            draw(&s, pass != 0, 1, false, 640);
            key(&s, CUI_KEY_TAB, 0); draw(&s, pass != 0, 1, false, 640);
            key(&s, CUI_KEY_ENTER, 0); draw(&s, pass != 0, 1, false, 640);
            cui_input_text(s.ui, invalid ? "7x" : "nan"); draw(&s, pass != 0, 1, false, 640);
            key(&s, CUI_KEY_ENTER, 0); draw(&s, pass != 0, 1, false, 640);
            CHECK(isfinite(s.number) && s.number == 12);
            destroy(&s);
        }
        sample s = create(false, 640);
        draw(&s, pass != 0, 2, false, 640);
        key(&s, CUI_KEY_TAB, 0); draw(&s, pass != 0, 2, false, 640);
        key(&s, CUI_KEY_A, CUI_MOD_PRIMARY); draw(&s, pass != 0, 2, false, 640);
        cui_input_key(s.ui, CUI_KEY_NONE, false, 0);
        char huge[64]; memset(huge, '9', 63); huge[63] = 0;
        cui_input_text(s.ui, huge); draw(&s, pass != 0, 2, false, 640);
        CHECK(strcmp(s.money, huge) == 0);
        destroy(&s);
    }
}

int main(void) {
    pixels(); inline_cancel_undo(); numeric_regressions();
    printf("Smart inputs: %d strict pixel comparisons; %d failures.\n", comparisons, failures);
    return failures ? 1 : 0;
}
