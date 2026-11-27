// [Documentation](../../../../docs/ui/gui/ai/agent-plan.md)
// [Prompt suggestions](../../../../docs/ui/gui/ai/prompt-suggestions.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

/** Compares immediate FPAS rail geometry with the settled CUI reference. */
static bool plan_rail_layout(void) {
    enum { width = 320, height = 420, bytes = width * height * 4 };
    static uint8_t first[bytes], wrapped[bytes], reference[bytes];
    for (int adapter = 0; adapter < 2; ++adapter) {
        cui_sw_renderer *renderer = cui_sw_create();
        if (!renderer) return false;
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_plan_step steps[] = {
            { "Find references", NULL, CUI_TASK_DONE },
            { "Check the format", NULL, CUI_TASK_SKIPPED },
            { "Read the answer", NULL, CUI_TASK_PENDING }
        };
        const char *detail = "A longer summary wraps across several lines when the plan stays at its original width.";
        bool match = true;
        for (int frame = 0; frame < 6; ++frame) {
            if (frame == 4) steps[0].detail = detail;
            cui_begin_frame(ui, .016f);
            if (adapter) {
                int nodes[3];
                fpas_cui_agent_plan(ui, "plan", steps, 3, NULL, 1, nodes);
            } else {
                cui_agent_plan(ui, "plan", steps, 3);
            }
            cui_end_frame(ui);
            if (!adapter && (frame == 3 || frame == 5)) {
                cui_sw_render(renderer, cui_get_draw_data(ui), frame == 3 ? first : wrapped, width, height, width * 4, CUI_SW_RGBA);
            } else if (adapter && (frame == 0 || frame == 4)) {
                cui_sw_render(renderer, cui_get_draw_data(ui), reference, width, height, width * 4, CUI_SW_RGBA);
                match = memcmp(reference, frame == 0 ? first : wrapped, bytes) == 0 && match;
            }
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
        if (!match) return false;
    }
    return true;
}

/** Exercises pointer-only activation and preservation of the passive plan. */
static bool plan_pointer_input(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 400, 300, 1);
    const cui_plan_step steps[] = { { "One", NULL, CUI_TASK_FAILED }, { "Two", "", CUI_TASK_PENDING } };
    cui_rect bounds = {0};
    bool valid = true;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1 || frame == 3)
            cui_input_mouse_move(ui, bounds.x + bounds.w * .5f, bounds.y + bounds.h * .5f);
        if (frame > 0) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, frame == 1 || frame == 3);
        cui_begin_frame(ui, .016f);
        int nodes[2];
        const int chosen = fpas_cui_agent_plan(ui, "steps", steps, 2, "", frame >= 3, nodes);
        const cui_id second = fpas_cui_node_key(ui, nodes[1]);
        cui_end_frame(ui);
        bounds = cui_get_rect(ui, second);
        valid = valid && chosen == (frame == 4 ? 1 : -1);
    }
    cui_destroy(ui);
    return valid;
}

/** Exercises focus, keyboard routing, resizing and retirement of a suggestion grid. */
static bool prompt_suggestion_input(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 560, 400, 1);
    cui_set_reduced_motion(ui, true);
    const cui_prompt_suggestion items[] = {
        { CUI_ICON_PENCIL, "First", NULL }, { "", "Second", "" }
    };
    const cui_key keys[] = {
        CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_ENTER,
        CUI_KEY_NONE, CUI_KEY_ENTER, CUI_KEY_SPACE, CUI_KEY_ESCAPE, CUI_KEY_SPACE,
        CUI_KEY_NONE, CUI_KEY_NONE, CUI_KEY_NONE, CUI_KEY_NONE, CUI_KEY_NONE
    };
    const int expected[] = { -1, -1, -1, 0, -1, -1, -1, -1, 0, -1, -1, -1, -1, 1 };
    bool valid = true;
    cui_rect target = {0};
    for (int frame = 0; frame < (int)(sizeof expected / sizeof *expected); ++frame) {
        if (keys[frame] != CUI_KEY_NONE) {
            cui_input_key(ui, keys[frame], true, 0);
            cui_input_key(ui, keys[frame], false, 0);
        }
        if (frame == 9) cui_set_viewport(ui, 330, 400, 1);
        if (frame == 11) cui_set_viewport(ui, 560, 400, 1);
        if (frame == 12) {
            cui_input_mouse_move(ui, target.x + target.w * .5f, target.y + target.h * .5f);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 13) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        cui_button(ui, "Popover anchor");
        if (frame == 4) cui_popover_open(ui, "Options");
        cui_popover(ui, "Options", .width = 150) { cui_text(ui, "Overlay"); }
        int nodes[2];
        const int count = frame == 10 ? 1 : 2;
        const int chosen = fpas_cui_prompt_suggestions(ui, "prompts", items, count, frame >= 11 ? 1 : 0,
            1, frame == 9 ? 1 : -1, nodes);
        valid = valid && chosen == expected[frame];
        if (frame == 5 || frame == 6) valid = valid && cui_key_pressed(ui, keys[frame]);
        const cui_id first = fpas_cui_node_key(ui, nodes[0]);
        const cui_id second = count > 1 ? fpas_cui_node_key(ui, nodes[1]) : 0;
        cui_end_frame(ui);
        if (frame == 0) valid = valid && cui_get_rect(ui, first).y == cui_get_rect(ui, second).y;
        if (frame == 9) {
            const cui_rect first_bounds = cui_get_rect(ui, first);
            target = cui_get_rect(ui, second);
            valid = valid && first_bounds.x == target.x && first_bounds.y < target.y && cui_focused_id(ui) == second;
        }
        if (frame == 10) valid = valid && cui_focused_id(ui) == 0;
        if (frame == 11) target = cui_get_rect(ui, second);
    }
    fpas_cui_forget(ui, cui_make_id(ui, "prompts"));
    valid = valid && cui_focused_id(ui) == 0;
    cui_destroy(ui);
    return valid;
}

/** Runs agent plan and prompt suggestion regressions. */
int main(void) {
    const bool plan_rail = plan_rail_layout();
    const bool plan_pointer = plan_pointer_input();
    const bool prompts = prompt_suggestion_input();
    if (!plan_rail) fprintf(stderr, "The plan rail did not track the current row layout.\n");
    if (!plan_pointer) fprintf(stderr, "The plan did not report pointer activation only when enabled.\n");
    if (!prompts) fprintf(stderr, "Unexpected prompt suggestion activation, layout, or focus.\n");
    return plan_rail && plan_pointer && prompts ? 0 : 1;
}
