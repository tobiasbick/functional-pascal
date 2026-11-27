// [Documentation](../../../../docs/ui/gui/canvas-and-bars/result-card.md)
// [Selection bar](../../../../docs/ui/gui/canvas-and-bars/selection-bar.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

static int failures;

static void check(bool condition, const char *message) {
    if (condition) return;
    fprintf(stderr, "%s\n", message);
    ++failures;
}

static cui_ctx *context(const cui_ctx_desc *desc) {
    cui_ctx *ui = cui_create(desc);
    cui_set_viewport(ui, 680, 550, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static bool result_card(cui_ctx *ui, const fpas_media_source *source, int sizing, float width,
    float aspect, bool enabled, bool stretch) {
    return fpas_cui_result_card(ui, "result", source, "Result", "Caption", "Done", CUI_TONE_WARNING,
        sizing, width, aspect, enabled, 0, stretch) != 0;
}

static cui_rect card_size(cui_ctx *ui, float parent_width, int sizing, float width, float aspect) {
    cui_begin_frame(ui, .016f);
    cui_column(ui, .width = CUI_PX(parent_width)) {
        result_card(ui, NULL, sizing, width, aspect, true, false);
    }
    cui_end_frame(ui);
    return cui_get_rect(ui, cui_make_id(ui, "result"));
}

/** A parent resize must schedule the frame that updates the card height. */
static void responsive_result(void) {
    cui_ctx *ui = context(NULL);
    card_size(ui, 240, 0, 0, 0);
    cui_rect r = card_size(ui, 240, 0, 0, 0);
    check(r.w == 240 && r.h == 300, "The default card must fit a narrow parent at aspect 1.25.");
    check(!cui_needs_redraw(ui), "A card at its settled size must stop requesting redraws.");

    card_size(ui, 600, 0, 0, 0);
    check(cui_needs_redraw(ui), "A wider parent must schedule a result card height update.");
    r = card_size(ui, 600, 0, 0, 0);
    check(r.w == 320 && r.h == 400, "The default card must cap its width at 320.");
    check(!cui_needs_redraw(ui), "A card at its capped width must stop requesting redraws.");

    card_size(ui, 180, 0, 0, 0);
    check(cui_needs_redraw(ui), "A narrower parent must schedule a result card height update.");
    r = card_size(ui, 180, 0, 0, 0);
    check(r.w == 180 && r.h == 225, "The card must shrink after its parent narrows.");
    card_size(ui, 600, 1, 0, .5f);
    r = card_size(ui, 600, 1, 0, .5f);
    check(r.w == 600 && r.h == 300, "Explicit fill must use the full width and supplied height ratio.");
    card_size(ui, 600, 3, 25, .5f);
    r = card_size(ui, 600, 3, 25, .5f);
    check(r.w == 150 && r.h == 75, "Percentage width must use the matching aspect height.");
    cui_destroy(ui);
}

static float image_span(cui_ctx *ui, const fpas_media_source *source, bool stretch) {
    cui_begin_frame(ui, .016f);
    result_card(ui, source, 2, 160, 1, true, stretch);
    cui_end_frame(ui);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    const cui_vertex *left = NULL, *right = NULL;
    for (int n = 0; n < draw->vertex_count; ++n) {
        const cui_vertex *v = &draw->vertices[n];
        if (v->mode != CUI_SHADE_IMAGE) continue;
        if (!left || v->x < left->x) left = v;
        if (!right || v->x > right->x) right = v;
    }
    if (!left || !right || left == right) return -1;
    /* Exclude the antialiasing gutter from the texture span. */
    return (right->u - left->u) * (2 * left->hw) / (right->x - left->x);
}

/** Compare cropped and stretched texture coordinates, then remove the image. */
static void result_image_modes(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = context(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    const uint8_t rgba[16] = {255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255};
    const fpas_media_source source = { .pixels = rgba, .width = 4, .height = 1, .version = 1 };
    check(fabsf(image_span(ui, &source, false) - .25f) < .001f, "A landscape image must crop to the square card.");
    check(fabsf(image_span(ui, &source, true) - 1) < .001f, "Stretch must show the full landscape image in the square card.");
    check(image_span(ui, NULL, false) < 0, "Removing the source must paint the card without an image texture.");
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}

static const cui_selection_action actions[] = {
    { CUI_ICON_MOVE, "Move", CUI_TONE_NEUTRAL }, { CUI_ICON_TRASH_2, "Delete", CUI_TONE_DANGER }
};

static int selection(cui_ctx *ui, bool *selected, int count, bool enabled, bool no_escape, const char *anchor) {
    return fpas_cui_selection_bar(ui, "bulk", selected, selected ? 3 : 0, count, actions, 2,
        enabled, no_escape, "%d chosen (100%%)", anchor);
}

static cui_id selection_part(cui_ctx *ui, int index, const char *name) {
    cui_push_id(ui, "bulk");
    if (index >= 0) cui_push_id_int(ui, index);
    const cui_id part = cui_make_id(ui, name);
    if (index >= 0) cui_pop_id(ui);
    cui_pop_id(ui);
    return part;
}

static int selection_frame(cui_ctx *ui, bool *selected, int count, bool enabled, bool no_escape,
    bool explicitly_anchored) {
    cui_begin_frame(ui, .016f);
    cui_box(ui, .id = "files", .width = CUI_PX(300), .height = CUI_PX(90)) {}
    if (explicitly_anchored) cui_box(ui, .id = "spacer", .width = CUI_PX(200), .height = CUI_PX(80)) {}
    const int result = selection(ui, selected, count, enabled, no_escape, explicitly_anchored ? "files" : NULL);
    cui_end_frame(ui);
    return result;
}

/** Flag mode uses the flags as the count. Explicit and implicit anchors place the same bar. */
static void selection_results(void) {
    cui_ctx *ui = context(NULL);
    bool flags[] = { true, false, true };
    check(selection_frame(ui, flags, 0, true, false, false) == CUI_SELECTION_NONE,
        "An idle selection bar must give no result.");
    selection_frame(ui, flags, 0, true, false, false);
    const cui_id clear_id = selection_part(ui, -1, "#clear");
    const cui_rect clear = cui_get_rect(ui, clear_id);
    const cui_rect files = cui_get_rect(ui, cui_make_id(ui, "files"));
    check(clear.w > 0 && clear.y > files.y + files.h, "The bar must anchor below the previous widget.");
    selection_frame(ui, flags, 0, true, false, true);
    check(cui_get_rect(ui, clear_id).y == clear.y, "An explicit anchor must override the previous widget.");

    const cui_rect remove = cui_get_rect(ui, selection_part(ui, 1, "Delete"));
    cui_input_mouse_move(ui, remove.x + remove.w / 2, remove.y + remove.h / 2);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    selection_frame(ui, flags, 0, true, false, true);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    check(selection_frame(ui, flags, 0, true, false, true) == 1 && flags[0] && flags[2],
        "Clicking Delete must report the zero-based index without clearing flags.");

    press(ui, CUI_KEY_ESCAPE);
    check(selection_frame(ui, flags, 0, true, true, false) == CUI_SELECTION_NONE && flags[0] && flags[2],
        "NoEscape must preserve selection.");
    press(ui, CUI_KEY_ESCAPE);
    check(selection_frame(ui, flags, 0, false, false, false) == CUI_SELECTION_NONE && flags[0] && flags[2],
        "A disabled bar must preserve selection.");
    press(ui, CUI_KEY_ESCAPE);
    check(selection_frame(ui, flags, 0, true, false, false) == CUI_SELECTION_CLEARED && !flags[0] && !flags[2],
        "Escape must clear the native flag buffer and report a clear proposal.");

    press(ui, CUI_KEY_ESCAPE);
    check(selection_frame(ui, NULL, 12, true, false, false) == CUI_SELECTION_CLEARED,
        "An aggregate count must report a clear proposal without item flags.");
    press(ui, CUI_KEY_ESCAPE);
    check(selection_frame(ui, NULL, 0, true, false, false) == CUI_SELECTION_NONE,
        "Escape must leave a zero count alone.");
    cui_destroy(ui);
}

/** Both activation keys must work before a popover opens and stay with the popover afterward. */
static void activation_keys(int focused_part) {
    cui_ctx *ui = context(NULL);
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_TAB,
        CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_NONE, CUI_KEY_ENTER, CUI_KEY_SPACE };
    for (int frame = 0; frame < 9; ++frame) {
        if ((frame <= focused_part || frame >= 4) && keys[frame] != CUI_KEY_NONE) press(ui, keys[frame]);
        cui_begin_frame(ui, .016f);
        cui_box(ui, .id = "files", .width = CUI_PX(300), .height = CUI_PX(55)) {}
        if (frame == 6) cui_popover_open(ui, "options");
        cui_popover(ui, "options", .width = 180) { cui_text(ui, "Overlay"); }
        const bool activated = result_card(ui, NULL, 2, 120, 0, true, false);
        const int result = selection(ui, NULL, 1, true, false, "files");
        if (frame == 4 || frame == 5) {
            const int expected = focused_part == 1 ? CUI_SELECTION_NONE : focused_part == 2 ? CUI_SELECTION_CLEARED : 0;
            check(activated == (focused_part == 1) && result == expected,
                "Enter and Space must activate the focused result card, clear control, or action.");
        } else if (frame == 7 || frame == 8) {
            check(!activated && result == CUI_SELECTION_NONE, "A popover must block the covered control's activation.");
            check(cui_key_pressed(ui, keys[frame]), "The bridge must leave the activation key available for the popover.");
        }
        cui_end_frame(ui);
    }
    cui_destroy(ui);
}

/** A pointer releases one activation, and disabling the card suppresses it. */
static void card_click(void) {
    cui_ctx *ui = context(NULL);
    int activations = 0;
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 2 || frame == 5) {
            const cui_rect r = cui_get_rect(ui, cui_make_id(ui, "result"));
            cui_input_mouse_move(ui, r.x + r.w / 2, r.y + r.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        } else if (frame == 3 || frame == 6) {
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        }
        cui_begin_frame(ui, .016f);
        activations += result_card(ui, NULL, 2, 140, 1, frame < 5, false);
        cui_end_frame(ui);
    }
    check(activations == 1, "A pointer must activate the enabled card once and leave the disabled card alone.");
    cui_destroy(ui);
}

/** Checks result and bulk-selection geometry, rendering modes, and input. */
int main(void) {
    responsive_result();
    result_image_modes();
    card_click();
    selection_results();
    for (int focused = 1; focused <= 3; ++focused) activation_keys(focused);
    return failures != 0;
}
