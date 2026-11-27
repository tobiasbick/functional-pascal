// [Documentation](../../../../docs/ui/gui/creative-controls/README.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static const cui_color focus_color = { 233, 21, 150, 255 };

static cui_ctx *context(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 400, 1);
    cui_set_reduced_motion(ui, true);
    cui_theme_get(ui)->colors[CUI_TOKEN_FOCUS_RING] = focus_color;
    return ui;
}

static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static int choices(cui_ctx *ui, bool aspect, int *index, int flags, int focus, int *nodes, bool tool) {
    if (aspect) {
        const int widths[] = { 1, 4 }, heights[] = { 1, 3 };
        const char *labels[] = { "", "Print" };
        return fpas_cui_aspect_picker(ui, "aspect", "Aspect ratio", index,
            widths, heights, labels, 2, "Poster", 720, NULL, flags | 2, focus, nodes);
    }
    const char *items[] = { "png", "jpg", "webp" };
    return fpas_cui_option_action(ui, "export", "Save as", items, 3, index,
        tool ? CUI_ICON_DOWNLOAD : NULL, "Download", "2.4 MB", flags, focus, nodes);
}

static cui_rect bounds(cui_ctx *ui, int node) {
    return cui_get_rect(ui, fpas_cui_node_key(ui, node));
}

static bool ring_at(cui_ctx *ui, cui_rect rect) {
    const cui_draw_data *draw = cui_get_draw_data(ui);
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_RRECT && memcmp(&v->border_color, &focus_color, sizeof focus_color) == 0 &&
            fabsf(v->x - v->lx - (rect.x + rect.w / 2)) < 1 &&
            fabsf(v->y - v->ly - (rect.y + rect.h / 2)) < 1)
            return true;
    }
    return false;
}

static bool directional_focus(bool aspect, cui_direction direction) {
    cui_ctx *ui = context();
    int index = 0, nodes[4], result = 0;
    bool ok = true;
    const bool rtl = direction == CUI_DIRECTION_RTL;
    const cui_key forward = rtl ? CUI_KEY_LEFT : CUI_KEY_RIGHT;
    const cui_key backward = rtl ? CUI_KEY_RIGHT : CUI_KEY_LEFT;
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, forward, forward, forward, backward };
    const int selected[] = { 0, 0, 1, 2, 2, 1 };
    const int changed[] = { 0, 0, 1, 1, 0, 1 };
    for (int frame = 0; frame < 6; ++frame) {
        if (frame) press(ui, keys[frame]);
        cui_begin_frame(ui, .016f);
        cui_column(ui, .width = CUI_FILL, .pad = 20, .direction = direction)
            result = choices(ui, aspect, &index, 1 | 16, -1, nodes, true);
        cui_end_frame(ui);
        const cui_rect first = bounds(ui, nodes[0]), second = bounds(ui, nodes[1]);
        ok &= index == selected[frame] && result == changed[frame] && (first.x > second.x) == rtl;
        if (frame)
            for (int part = 0; part < 3; ++part) {
                const bool focused = part == selected[frame];
                ok &= (fpas_cui_focused(ui, nodes[part]) != 0) == focused;
                ok &= ring_at(ui, bounds(ui, nodes[part])) == focused;
            }
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "%s keyboard selection and painted focus disagree in %s layout.\n",
        aspect ? "Aspect picker" : "Option action", rtl ? "RTL" : "LTR");
    return ok;
}

static bool popup_keys(bool aspect, int focus, cui_key key) {
    cui_ctx *ui = context();
    const int initial = key == CUI_KEY_RIGHT ? 2 : key == CUI_KEY_LEFT ? 0 : 1;
    int index = initial, nodes[4];
    cui_begin_frame(ui, .016f);
    choices(ui, aspect, &index, 1 | 16, focus, nodes, true);
    cui_end_frame(ui);
    press(ui, CUI_KEY_TAB);
    cui_begin_frame(ui, .016f);
    choices(ui, aspect, &index, 1 | 16, focus, nodes, true);
    cui_end_frame(ui);
    bool ok = fpas_cui_focused(ui, nodes[focus]) != 0;
    press(ui, key);
    cui_begin_frame(ui, .016f);
    cui_popover_open(ui, "Cover");
    const int covered = choices(ui, aspect, &index, 1 | 16, -1, nodes, true);
    bool popup_has_key = false;
    cui_popover(ui, "Cover") {
        popup_has_key = cui_key_pressed(ui, key);
        cui_text(ui, "Cover");
    }
    cui_end_frame(ui);
    ok &= covered == 0 && index == initial && popup_has_key;
    press(ui, CUI_KEY_ESCAPE);
    cui_begin_frame(ui, .016f);
    cui_popover(ui, "Cover") { cui_text(ui, "Cover"); }
    choices(ui, aspect, &index, 1 | 16, focus, nodes, true);
    cui_end_frame(ui);
    press(ui, key);
    cui_begin_frame(ui, .016f);
    const int uncovered = choices(ui, aspect, &index, 1 | 16, -1, nodes, true);
    cui_end_frame(ui);
    ok &= uncovered == (focus == 3 ? CUI_WELL_ACTION : CUI_WELL_CHANGED);
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "%s part %d handled key %d under a popup or lost it after dismissal.\n",
        aspect ? "Aspect picker" : "Option action", focus, key);
    return ok;
}

static bool optional_action(void) {
    cui_ctx *ui = context();
    int index = 0, nodes[4];
    cui_begin_frame(ui, .016f);
    choices(ui, false, &index, 1 | 4 | 8 | 16, 3, nodes, true);
    cui_end_frame(ui);
    bool ok = fpas_cui_focused(ui, nodes[3]) != 0;
    press(ui, CUI_KEY_ENTER);
    cui_begin_frame(ui, .016f);
    const int disabled = choices(ui, false, &index, 1 | 4 | 8, 3, nodes, true);
    cui_end_frame(ui);
    ok &= disabled == 0 && index == 0 && nodes[3] >= 0 && !fpas_cui_focused(ui, nodes[3]);
    cui_begin_frame(ui, .016f);
    choices(ui, false, &index, 1 | 8 | 16, 3, nodes, true);
    cui_end_frame(ui);
    press(ui, CUI_KEY_SPACE);
    cui_begin_frame(ui, .016f);
    const int absent = choices(ui, false, &index, 1 | 8 | 16, 3, nodes, false);
    cui_end_frame(ui);
    ok &= absent == 0 && nodes[3] == -1 && cui_focused_id(ui) == 0;
    press(ui, CUI_KEY_TAB);
    cui_begin_frame(ui, .016f);
    choices(ui, false, &index, 1 | 8 | 16, -1, nodes, false);
    cui_end_frame(ui);
    ok &= fpas_cui_focused(ui, nodes[0]) != 0;
    press(ui, CUI_KEY_RIGHT);
    cui_begin_frame(ui, .016f);
    const int changed = choices(ui, false, &index, 1 | 8 | 16, -1, nodes, false);
    cui_end_frame(ui);
    ok &= changed == CUI_WELL_CHANGED && index == 1 && nodes[3] == -1;
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Option action did not keep selection and focus when its action was disabled or absent.\n");
    return ok;
}

int main(void) {
    bool ok = optional_action();
    const cui_key keys[] = { CUI_KEY_LEFT, CUI_KEY_RIGHT, CUI_KEY_ENTER, CUI_KEY_SPACE };
    for (int aspect = 0; aspect < 2; ++aspect) {
        ok &= directional_focus(aspect != 0, CUI_DIRECTION_LTR);
        ok &= directional_focus(aspect != 0, CUI_DIRECTION_RTL);
        for (int i = 0; i < 4; ++i) {
            const int focus = keys[i] == CUI_KEY_LEFT ? 2 : 0;
            ok &= popup_keys(aspect != 0, focus, keys[i]);
        }
    }
    ok &= popup_keys(false, 3, CUI_KEY_ENTER);
    ok &= popup_keys(false, 3, CUI_KEY_SPACE);
    return ok ? 0 : 1;
}
