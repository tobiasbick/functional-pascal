// [Documentation](../../../../docs/ui/gui/native-build.md#composite-and-popup-adapters)
// [Confirm popover](../../../../docs/ui/gui/disclosure/confirm-popover.md)
/* Compare adapter output to the actual CUI widgets, including omitted decorations. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { cui_rect link, panel; bool activated, open; } hover_frame;

/** Draw the link followed by a hover card above its native anchor. */
static hover_frame draw_hover_card(cui_ctx *ui, float elapsed, bool enabled) {
    int link = -1, panel = -1;
    unsigned int owner;
    hover_frame frame = {0};
    cui_begin_frame(ui, elapsed);
    cui_column(ui, .pad = 70) {
        frame.activated = fpas_cui_link(ui, "###link", "@jonas", 1, 0, &link) != 0;
        if (fpas_cui_hover_card_begin(ui, "###details", CUI_PLACE_TOP_START, 210,
            0.15f, enabled, &panel, &owner)) {
            cui_text(ui, "Profile details");
            fpas_cui_hover_card_end(ui, link);
        }
    }
    cui_end_frame(ui);
    frame.link = fpas_cui_node_rect(ui, link);
    frame.open = panel >= 0;
    if (frame.open) frame.panel = fpas_cui_node_rect(ui, panel);
    return frame;
}

/** Check delay, placement, pointer release, Escape and disabling without using the host clock. */
static bool check_hover_card(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 420, 300, 1);
    cui_set_reduced_motion(ui, true);
    hover_frame frame = draw_hover_card(ui, 0.04f, true);
    const float x = frame.link.x + 4, y = frame.link.y + 4;
    cui_input_mouse_move(ui, x, y);
    bool valid = !draw_hover_card(ui, 0.04f, true).open;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    valid &= !draw_hover_card(ui, 0.22f, true).open && cui_next_wakeup(ui) < 0;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    frame = draw_hover_card(ui, 0.04f, true);
    valid &= frame.activated && frame.open && frame.panel.w == 210
        && frame.panel.y + frame.panel.h < frame.link.y;
    cui_input_mouse_move(ui, frame.panel.x + 4, frame.panel.y + 4);
    valid &= draw_hover_card(ui, 0.24f, true).open;
    cui_input_mouse_move(ui, x, y);
    cui_input_key(ui, CUI_KEY_ESCAPE, true, 0);
    cui_input_key(ui, CUI_KEY_ESCAPE, false, 0);
    valid &= !draw_hover_card(ui, 0.04f, true).open;
    valid &= !draw_hover_card(ui, 0.22f, true).open;
    cui_input_mouse_move(ui, 400, 280);
    draw_hover_card(ui, 0.04f, true);
    cui_input_mouse_move(ui, x, y);
    valid &= !draw_hover_card(ui, 0.04f, true).open;
    valid &= draw_hover_card(ui, 0.22f, true).open;
    valid &= !draw_hover_card(ui, 0.04f, false).open;
    valid &= !draw_hover_card(ui, 0.04f, true).open;
    valid &= draw_hover_card(ui, 0.22f, true).open;
    cui_input_mouse_move(ui, 400, 280);
    valid &= draw_hover_card(ui, 0.04f, true).open;
    valid &= !draw_hover_card(ui, 0.24f, true).open;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Hover-card timing, placement or trigger activation mismatch.\n");
    return valid;
}

static bool pixels_match(int kind, bool open, bool decorated, bool dark, int width) {
    const int height = 440;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
                if (kind == 2) {
                    const char *text = "Native text wraps across several lines. Literal ## hashes are retained. "
                        "Native text wraps across several lines. Native text wraps across several lines. "
                        "Native text wraps across several lines. Native text wraps across several lines.";
                    const cui_expandable_text_opts options = { .lines = 2,
                        .size = decorated ? 18 : 0,
                        .color = decorated ? CUI_RGB(0x21435d) : (cui_color){0},
                        .fade_to = decorated ? CUI_RGB(0xaaddee) : (cui_color){0} };
                    if (pass) {
                        int toggle;
                        fpas_cui_expandable_text(ui, "text", text, 2, options.size, decorated ? 3 : 0,
                            decorated ? 0xff5d4321u : 0, decorated ? 0xffeeddaau : 0, 1, 0, &toggle);
                    } else cui_expandable_text_ex(ui, "text", text, &options);
                } else {
                    bool value = open;
                    const cui_expandable_card_opts options = {
                        .description = decorated ? "Supporting description" : NULL,
                        .icon = decorated ? CUI_ICON_CREDIT_CARD : NULL,
                        .badge = decorated ? "Active" : NULL, .badge_tone = CUI_TONE_SUCCESS
                    };
                    int node, header;
                    bool draw;
                    if (pass) draw = (fpas_cui_disclosure_begin(ui, "###header", "Native title", kind,
                        open, 1, 0, options.description ? options.description : "",
                        options.icon ? options.icon : "", options.badge ? options.badge : "",
                        CUI_TONE_SUCCESS, &node, &header) & 1) != 0;
                    else draw = kind ? cui_expandable_card_begin(ui, "Native title", &value, &options)
                                     : cui_accordion_begin(ui, "Native title", &value);
                    if (draw) {
                        cui_text(ui, "The same native body.");
                        cui_button(ui, "Body action");
                        if (pass) fpas_cui_disclosure_end(ui, kind, node);
                        else if (kind) cui_expandable_card_end(ui);
                        else cui_accordion_end(ui);
                    }
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Disclosure pixel mismatch: kind=%d open=%d decorated=%d dark=%d width=%d\n",
        kind, open, decorated, dark, width);
    return valid;
}

static bool confirm_pixels_match(int placement, bool decorated, bool dark, int width) {
    const int height = 440;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool valid = pixels[0] && pixels[1];
    const cui_confirm_popover_opts options = {
        .title = decorated ? "Delete feature/grid?" : NULL,
        .description = decorated ? "The example dismisses its preview." : NULL,
        .confirm_label = decorated ? "Delete branch" : NULL,
        .cancel_label = decorated ? "Keep branch" : NULL,
        .danger = decorated,
        .placement = (cui_placement)placement
    };
    for (int pass = 0; pass < 2 && valid; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = renderer ? cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)}) : NULL;
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .04f);
            cui_column(ui, .width = CUI_FILL, .pad = 80) {
                cui_button(ui, "Delete branch", .icon = CUI_ICON_TRASH_2);
                if (pass) {
                    int nodes[3];
                    unsigned int owner;
                    const int action = fpas_cui_confirm_popover(ui, "question", options.title, options.description,
                        options.confirm_label ? options.confirm_label : "Confirm",
                        options.cancel_label ? options.cancel_label : "Cancel",
                        placement, options.danger, frame > 0, 1, nodes, &owner);
                    valid = valid && action == (frame > 0 ? 4 : 0);
                } else {
                    if (frame == 1) cui_popover_open(ui, "question");
                    valid = !cui_confirm_popover_ex(ui, "question", &options) && valid;
                }
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]);
    free(pixels[1]);
    if (!valid) fprintf(stderr, "Confirm popover pixel mismatch: placement=%d decorated=%d dark=%d width=%d\n",
        placement, decorated, dark, width);
    return valid;
}

typedef struct {
    int action;
    bool next;
    cui_id anchor, cancel, confirm, other;
} confirm_frame;

static confirm_frame draw_confirmation(cui_ctx *ui, bool open, cui_key key, int modifiers) {
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, modifiers);
        cui_input_key(ui, key, false, modifiers);
    }
    cui_begin_frame(ui, .04f);
    cui_button(ui, "Open preview");
    confirm_frame result = { .anchor = cui_last_id(ui) };
    int nodes[3];
    unsigned int owner;
    result.action = fpas_cui_confirm_popover(ui, "question", NULL, "Preview action", "Confirm", "Cancel",
        CUI_PLACE_BOTTOM_START, 1, open, 1, nodes, &owner);
    result.cancel = fpas_cui_node_key(ui, nodes[1]);
    result.confirm = fpas_cui_node_key(ui, nodes[2]);
    result.next = cui_button(ui, "Other action");
    result.other = cui_last_id(ui);
    cui_end_frame(ui);
    return result;
}

static bool confirm_uses_focused_activation(void) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){.no_default_fonts = true});
    if (!ui) return false;
    cui_set_viewport(ui, 640, 440, 1);
    draw_confirmation(ui, false, CUI_KEY_NONE, 0);
    const confirm_frame trigger = draw_confirmation(ui, false, CUI_KEY_TAB, 0);
    bool valid = cui_focused_id(ui) == trigger.anchor;
    const confirm_frame open = draw_confirmation(ui, true, CUI_KEY_ENTER, 0);
    valid = valid && open.action == 4 && cui_focused_id(ui) == open.confirm;
    for (int i = 0; i < 12; ++i) draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    const confirm_frame tab = draw_confirmation(ui, true, CUI_KEY_TAB, 0);
    valid = valid && cui_focused_id(ui) == tab.other;
    const confirm_frame other = draw_confirmation(ui, true, CUI_KEY_ENTER, 0);
    valid = valid && other.action == 4 && other.next;
    const confirm_frame escape_outside = draw_confirmation(ui, true, CUI_KEY_ESCAPE, 0);
    valid = valid && escape_outside.action == 2 && cui_focused_id(ui) == other.other;

    draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    for (int i = 0; i < 12; ++i) draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    const confirm_frame back = draw_confirmation(ui, true, CUI_KEY_TAB, CUI_MOD_SHIFT);
    valid = valid && cui_focused_id(ui) == back.cancel;
    const confirm_frame cancel = draw_confirmation(ui, true, CUI_KEY_ENTER, 0);
    valid = valid && cancel.action == 2 && cui_focused_id(ui) == trigger.anchor;

    draw_confirmation(ui, false, CUI_KEY_NONE, 0);
    draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    for (int i = 0; i < 12; ++i) draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    const confirm_frame escape = draw_confirmation(ui, true, CUI_KEY_ESCAPE, 0);
    valid = valid && escape.action == 2 && cui_focused_id(ui) == trigger.anchor;

    draw_confirmation(ui, false, CUI_KEY_NONE, 0);
    draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    draw_confirmation(ui, true, CUI_KEY_NONE, 0);
    const confirm_frame confirmed = draw_confirmation(ui, true, CUI_KEY_ENTER, 0);
    valid = valid && confirmed.action == 1 && cui_focused_id(ui) == trigger.anchor;
    valid = draw_confirmation(ui, false, CUI_KEY_NONE, 0).action == 0 && valid;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Confirm popover keyboard activation or focus mismatch.\n");
    return valid;
}

int main(void) {
    bool valid = true;
    for (int kind = 0; kind < 3; ++kind)
        for (int open = 0; open < 2; ++open)
            for (int decorated = 0; decorated < 2; ++decorated)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = pixels_match(kind, open != 0, decorated != 0, dark != 0,
                            narrow ? 280 : 640) && valid;
    for (int placement = CUI_PLACE_BOTTOM_START; placement <= CUI_PLACE_LEFT; ++placement)
        for (int decorated = 0; decorated < 2; ++decorated)
            for (int dark = 0; dark < 2; ++dark)
                valid = confirm_pixels_match(placement, decorated != 0, dark != 0,
                    dark ? 320 : 640) && valid;
    valid = confirm_uses_focused_activation() && valid;
    valid = check_hover_card() && valid;
    return valid ? 0 : 1;
}
