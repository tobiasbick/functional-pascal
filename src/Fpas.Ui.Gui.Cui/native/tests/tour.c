// [Documentation](../../../../docs/ui/gui/overlays/guided-tour.md)
/* The binding must preserve native tour painting and leave its base view's last node intact. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

/** Compares native options and the adapter's action locations and pixels. */
static bool matches(int selected, bool target, bool dark, bool block, int width, bool blank) {
    const int height = 420;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&options);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .04f);
            cui_column(ui, .width = CUI_FILL, .pad = 24) { cui_skeleton(ui, CUI_PX(80), 80); }
            const int saved = fpas_cui_last_node(ui);
            const unsigned int key = target ? cui_last_id(ui) : 0;
            const unsigned int targets[3] = { key, key, key };
            const char *titles[3] = { "Turn ## literally", "Watch the levels", "Ride the faders" };
            const char *bodies[3] = { "Drag up and down. Shift gives fine steps.", "Native peak hold.", "Use the wheel or arrow keys." };
            const int positions[3] = { CUI_PLACE_BOTTOM_START, CUI_PLACE_TOP_START, CUI_PLACE_LEFT_START };
            const char *next = blank ? "" : block ? "Continue" : NULL;
            const char *back = blank ? "" : block ? "Previous" : NULL;
            const char *done = blank ? "" : block ? "Finish" : NULL;
            const char *format = blank ? "" : block ? "Step %d / %d" : NULL;
            if (frame > 0) {
                int step = selected;
                if (pass) {
                    int node, parts[3]; unsigned int owner;
                    fpas_cui_tour(ui, "###guide", targets, titles, bodies, positions, 3, &step,
                        block ? 260 : 0, block ? 9 : 0, block, next, back, done, format, 0, &node, &owner, parts);
                    valid = valid && node >= 0 && parts[0] >= 0 && parts[2] >= 0 &&
                        (parts[1] >= 0) == (selected > 0) && owner == cui_make_id(ui, "###guide") &&
                        fpas_cui_last_node(ui) == saved;
                } else {
                    const cui_tour_step steps[3] = {
                        { key, titles[0], bodies[0], (cui_placement)positions[0] },
                        { key, titles[1], bodies[1], (cui_placement)positions[1] },
                        { key, titles[2], bodies[2], (cui_placement)positions[2] }
                    };
                    const cui_tour_opts settings = { .width = block ? 260 : 0, .padding = block ? 9 : 0, .block_target = block,
                        .next_label = next, .back_label = back, .done_label = done, .count_format = format };
                    cui_tour_ex(ui, "###guide", steps, 3, &step, &settings);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Tour mismatch: step=%d target=%d dark=%d block=%d width=%d\n", selected, target, dark, block, width);
    return valid;
}

/** Sends one complete key press to CUI. */
static void tap(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

/** Moves keyboard focus into the tour, through its target, and back to the preceding control. */
static bool keyboard_navigation(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 700, 450, 1);
    cui_set_reduced_motion(ui, true);
    const char *titles[] = { "Fader", "Next" }, *bodies[] = { "Try it", "" };
    const int placements[] = { CUI_PLACE_RIGHT, CUI_PLACE_BOTTOM };
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_NONE, CUI_KEY_RIGHT, CUI_KEY_LEFT,
        CUI_KEY_NONE, CUI_KEY_NONE, CUI_KEY_NONE, CUI_KEY_UP, CUI_KEY_RIGHT, CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_ESCAPE };
    const int events[] = { CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NEXT, CUI_TOUR_BACK,
        CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_NONE, CUI_TOUR_SKIPPED };
    bool valid = true;
    float covered = .5f, highlighted = .5f;
    int step = 0;
    cui_rect target = {0};
    for (int frame = 0; frame < (int)(sizeof keys / sizeof *keys); ++frame) {
        if (keys[frame] != CUI_KEY_NONE) tap(ui, keys[frame]);
        if (frame == 6) {
            cui_input_mouse_move(ui, target.x + target.w * .5f, target.y + target.h * .5f);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 7) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 10) cui_input_scroll(ui, 0, 1);
        cui_begin_frame(ui, .016f);
        cui_fader(ui, "Obscured", &covered, 0, 1, .step = .1f, .horizontal = true, .length = 140);
        const cui_id other = cui_last_id(ui);
        cui_fader(ui, "Highlighted", &highlighted, 0, 1, .step = .1f, .horizontal = true, .length = 140);
        const cui_id selected = cui_last_id(ui);
        int event = CUI_TOUR_NONE;
        cui_id next = 0, close = 0;
        if (frame >= 2) {
            const unsigned int targets[] = { selected, 0 };
            int node, parts[3]; unsigned int owner;
            event = fpas_cui_tour(ui, "tour", targets, titles, bodies, placements, 2, &step,
                0, 0, 0, NULL, NULL, NULL, NULL, 0, &node, &owner, parts);
            next = fpas_cui_node_key(ui, parts[2]);
            close = fpas_cui_node_key(ui, parts[0]);
            valid = valid && cui_last_id(ui) == selected;
        }
        cui_end_frame(ui);
        target = cui_get_rect(ui, selected);
        const cui_id wanted = frame == 0 ? 0 : frame == 1 || frame == 12 ? other :
            frame == 11 ? close : frame < 6 ? next : selected;
        const float gain = frame < 8 ? .5f : frame == 8 ? .6f : frame == 9 ? .7f : .8f;
        if (event != events[frame] || cui_focused_id(ui) != wanted || fabsf(covered - .5f) > .001f ||
            fabsf(highlighted - gain) > .001f) {
            fprintf(stderr, "Tour frame %d: action %d, focus %u expected %u, faders %.2f %.2f\n",
                frame, event, cui_focused_id(ui), wanted, covered, highlighted);
            valid = false;
        }
    }
    cui_destroy(ui);
    return valid && step == -1;
}

/** Routes the wheel only through an open spotlight. */
static bool spotlight_wheel(bool block) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 720, 420, 1);
    cui_set_reduced_motion(ui, true);
    const char *titles[] = { "Spotlight" }, *bodies[] = { "" };
    const int placements[] = { CUI_PLACE_RIGHT };
    cui_rect target = {0};
    bool valid = true;
    for (int frame = 0; frame < 4; ++frame) {
        if (frame >= 2) {
            cui_input_mouse_move(ui, target.x + target.w / 2,
                frame == 2 ? target.y + target.h + 28 : target.y + target.h / 2);
            cui_input_scroll(ui, 0, -1);
        }
        cui_begin_frame(ui, .016f);
        cui_id anchor = 0;
        cui_scroll(ui, .id = "Scroll", .width = CUI_PX(250), .height = CUI_PX(160)) {
            cui_button(ui, "Spotlight target");
            anchor = cui_last_id(ui);
            cui_skeleton(ui, CUI_PX(180), 280);
        }
        if (frame > 0) {
            int step = 0, node, parts[3]; unsigned int owner;
            fpas_cui_tour(ui, "wheel-tour", &anchor, titles, bodies, placements, 1, &step,
                0, 0, block, NULL, NULL, NULL, NULL, 0, &node, &owner, parts);
        }
        cui_end_frame(ui);
        const cui_rect next = cui_get_rect(ui, anchor);
        if (frame >= 2 && (frame == 2 || block ? next.y != target.y : next.y >= target.y)) {
            fprintf(stderr, "Tour wheel frame %d: target y from %g to %g\n", frame, target.y, next.y);
            valid = false;
        }
        target = next;
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "The tour routed wheel input incorrectly (block %d).\n", block);
    return valid;
}

/** Runs native tour option, pixel and input comparisons. */
int main(void) {
    bool valid = true;
    for (int step = 0; step < 3; ++step)
        for (int target = 0; target < 2; ++target)
            for (int dark = 0; dark < 2; ++dark)
                for (int block = 0; block < 2; ++block)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = matches(step, target != 0, dark != 0, block != 0, narrow ? 280 : 640, false) && valid;
    for (int step = 0; step < 3; ++step)
        valid = matches(step, true, false, false, 640, true) && valid;
    const bool keys = keyboard_navigation(), wheel = spotlight_wheel(false), blocked = spotlight_wheel(true);
    return valid && keys && wheel && blocked ? 0 : 1;
}
