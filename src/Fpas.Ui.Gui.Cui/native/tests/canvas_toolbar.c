// [Documentation](../../../../docs/ui/gui/canvas-and-bars/toolbar.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static int failures;

static void check(bool value, const char *message) {
    if (value) return;
    fprintf(stderr, "%s\n", message);
    failures++;
}

static cui_ctx *canvas(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return NULL;
    cui_set_viewport(ui, 650, 440, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void toolbar_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static void toolbar_cycle_and_primary(void) {
    cui_ctx *ui = canvas();
    const char *labels[] = { "PNG", "JPEG" };
    int selected = 0;
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_ENTER,
        CUI_KEY_SPACE, CUI_KEY_TAB, CUI_KEY_ENTER, CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_ESCAPE, CUI_KEY_SPACE };
    for (int frame = 0; frame < (int)(sizeof keys / sizeof *keys); ++frame) {
        if (keys[frame]) toolbar_key(ui, keys[frame]);
        cui_begin_frame(ui, .016f);
        if (frame == 7) cui_popover_open(ui, "Cover");
        cui_popover(ui, "Cover") {
            check(cui_key_pressed(ui, keys[frame]), "The covering popup must receive the activation key.");
            cui_text(ui, "Cover");
        }
        fpas_cui_toolbar_begin(ui, "tools", CUI_TOOLBAR_GROUPED, 0, 1);
        const int next = fpas_cui_toolbar_cycle(ui, "cycle", labels, frame == 2 ? 1 : 2, selected,
            "Format", "Ctrl F", CUI_TONE_DANGER, frame == 4 ? 3 : 2);
        const cui_id cycle = cui_last_id(ui);
        const int primary = fpas_cui_toolbar_button(ui, CUI_ICON_SPARKLES, "Generate", NULL, NULL,
            1, CUI_TONE_WARNING, 4);
        fpas_cui_toolbar_end(ui);
        cui_end_frame(ui);

        check(next == (frame == 3 ? 1 : -1), "Cycle must propose only a changed selection while enabled.");
        check(primary == (frame == 6 || frame == 10 ? 0 : -1),
            "A primary toolbar action must propose zero when activated and leave keys to a covering popup.");
        if (next >= 0) selected = next;
        if (frame == 0) {
            const cui_color danger = cui_theme_get(ui)->colors[CUI_TOKEN_DANGER];
            const cui_color muted = cui_theme_get(ui)->colors[CUI_TOKEN_TEXT_MUTED];
            const cui_rect bounds = cui_get_rect(ui, cycle);
            const cui_draw_data *draw = cui_get_draw_data(ui);
            bool colored = false, marker = false;
            for (int i = 0; i < draw->vertex_count; ++i) {
                const cui_vertex *v = &draw->vertices[i];
                colored |= v->mode == CUI_SHADE_TEXT && memcmp(&v->color, &danger, sizeof danger) == 0;
                marker |= v->mode == CUI_SHADE_SOLID && v->color.r == muted.r && v->color.g == muted.g && v->color.b == muted.b &&
                    fabsf(v->x - (bounds.x + bounds.w - 5)) < 1 && fabsf(v->y - (bounds.y + bounds.h - 5)) < 1;
            }
            check(colored && marker, "Cycle tone and flyout must reach the draw data.");
        }
    }
    cui_destroy(ui);
}

static const char *const toolbar_icons[] = { CUI_ICON_MOVE, CUI_ICON_SQUARE_PEN, CUI_ICON_TYPE };

static void toolbar_overflow(bool enabled) {
    cui_ctx *ui = canvas();
    const char *labels[] = { "Move", "Draw", "Text" };
    for (int frame = 0; frame < 14; ++frame) {
        if (frame == 4) toolbar_key(ui, CUI_KEY_TAB);
        if (frame == 5 || frame == 10) toolbar_key(ui, CUI_KEY_ENTER);
        if (frame == 8 || frame == 9) toolbar_key(ui, CUI_KEY_DOWN);
        cui_begin_frame(ui, .016f);
        cui_box(ui, .width = CUI_PX(85)) {
            fpas_cui_toolbar_begin(ui, "tools", CUI_TOOLBAR_GROUPED, 0, enabled || frame != 11);
            const int next = fpas_cui_toolbar_radio(ui, "radio", toolbar_icons, labels, 3, 0);
            check(next == (enabled && frame == 11 ? 1 : -1),
                "Overflow radio must deliver once on the next frame and discard its choice when disabled.");
            fpas_cui_toolbar_end(ui);
        }
        cui_end_frame(ui);
    }
    cui_destroy(ui);
}

static bool toolbar_ring_at(cui_ctx *ui, cui_rect rect, cui_color color) {
    const cui_draw_data *draw = cui_get_draw_data(ui);
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_RRECT && memcmp(&v->border_color, &color, sizeof color) == 0 &&
            fabsf(v->x - v->lx - (rect.x + rect.w * .5f)) < 1 &&
            fabsf(v->y - v->ly - (rect.y + rect.h * .5f)) < 1) return true;
    }
    return false;
}

static void toolbar_radio_direction(cui_direction direction) {
    cui_ctx *ui = canvas();
    const cui_color focus = { 233, 21, 150, 255 };
    cui_theme_get(ui)->colors[CUI_TOKEN_FOCUS_RING] = focus;
    const bool rtl = direction == CUI_DIRECTION_RTL;
    const cui_key forward = rtl ? CUI_KEY_LEFT : CUI_KEY_RIGHT;
    cui_id first = 0;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1) toolbar_key(ui, CUI_KEY_TAB);
        if (frame == 2 || frame == 4) toolbar_key(ui, forward);
        if (frame == 3) toolbar_key(ui, CUI_KEY_ESCAPE);
        cui_begin_frame(ui, .016f);
        if (frame == 2) cui_popover_open(ui, "Cover");
        cui_popover(ui, "Cover") {
            check(cui_key_pressed(ui, forward), "The covering popup must receive the arrow key.");
            cui_text(ui, "Cover");
        }
        cui_column(ui, .width = CUI_FILL, .direction = direction) {
            fpas_cui_toolbar_begin(ui, "tools", CUI_TOOLBAR_GROUPED, 0, 1);
            const int next = fpas_cui_toolbar_radio(ui, "radio", toolbar_icons, NULL, 3, 0);
            check(next == (frame == 4 ? 1 : -1), "Radio arrow must follow the visual direction after popup dismissal.");
            fpas_cui_toolbar_end(ui);
        }
        cui_end_frame(ui);
        if (frame == 1) first = cui_focused_id(ui);
        if (frame == 4) {
            const cui_rect old = cui_get_rect(ui, first), next = cui_get_rect(ui, cui_focused_id(ui));
            check(first && (rtl ? next.x < old.x : next.x > old.x) &&
                !toolbar_ring_at(ui, old, focus) && toolbar_ring_at(ui, next, focus),
                "The focus ring must move to the next radio icon in the same frame.");
        }
    }
    cui_destroy(ui);
}

int main(void) {
    toolbar_cycle_and_primary();
    toolbar_overflow(true);
    toolbar_overflow(false);
    toolbar_radio_direction(CUI_DIRECTION_LTR);
    toolbar_radio_direction(CUI_DIRECTION_RTL);
    return failures != 0;
}
