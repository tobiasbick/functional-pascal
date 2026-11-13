#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static bool render_composer(cui_ctx *ui, char *draft, int flags, int focus, int *nodes) {
    int removed = -1, selected = 0;
    const int events = fpas_cui_composer(ui, "composer", draft, 128, "Message", NULL, 0, NULL, 0, &selected,
        NULL, 0, 0, 0, 0, 0, flags, focus, &removed, nodes);
    return (events & CUI_COMPOSER_SEND) != 0;
}

static bool composer_covers_keyboard(cui_ctx *ui) {
    char draft[128] = "Hello";
    int nodes[4];
    cui_begin_frame(ui, 0.05f);
    render_composer(ui, draft, 1, 0, nodes);
    cui_end_frame(ui);

    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, CUI_MOD_SHIFT);
    cui_input_key(ui, CUI_KEY_ENTER, false, CUI_MOD_SHIFT);
    const bool submitted = render_composer(ui, draft, 1, -1, nodes);
    cui_end_frame(ui);
    if (submitted || !strchr(draft, '\n')) return false;

    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const bool disabled_submitted = render_composer(ui, draft, 0, -1, nodes);
    cui_end_frame(ui);
    if (disabled_submitted) return false;

    cui_begin_frame(ui, 0.05f);
    render_composer(ui, draft, 1, 0, nodes);
    cui_end_frame(ui);
    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const bool active_submitted = render_composer(ui, draft, 1, -1, nodes);
    cui_end_frame(ui);
    return active_submitted;
}

static bool model_menu_selects_with_enter(cui_ctx *ui) {
    const cui_model models[] = { { "First", "One", NULL, NULL }, { "Second", "Two", NULL, NULL } };
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_ENTER, CUI_KEY_DOWN, CUI_KEY_DOWN, CUI_KEY_ENTER };
    int selected = 0;
    for (int frame = 0; frame < 5; ++frame) {
        cui_begin_frame(ui, 0.05f);
        if (keys[frame] != CUI_KEY_NONE) {
            cui_input_key(ui, keys[frame], true, 0);
            cui_input_key(ui, keys[frame], false, 0);
        }
        fpas_cui_model_picker(ui, "models", models, 2, &selected, CUI_PLACE_BOTTOM, frame == 0 ? 3 : 1);
        cui_end_frame(ui);
    }
    return selected == 1;
}

static bool status_stops_when_motion_is_reduced(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    for (int frame = 0; frame < 12; ++frame) {
        cui_begin_frame(ui, 0.05f);
        const cui_plan_step step = { "Reading", NULL, CUI_TASK_RUNNING };
        int node;
        if (adapter) fpas_cui_agent_plan(ui, "job", &step, 1, NULL, 1, &node);
        else cui_agent_plan(ui, "job", &step, 1);
        cui_end_frame(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    return stopped;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return status_stops_when_motion_is_reduced(false) ? 0 : 1;
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool composer = composer_covers_keyboard(ui);
    cui_destroy(ui);
    ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool models = model_menu_selects_with_enter(ui);
    cui_destroy(ui);
    if (!composer) fprintf(stderr, "Composer did not distinguish Enter from Shift+Enter and disabled input.\n");
    if (!models) fprintf(stderr, "The model menu did not commit its highlighted model.\n");
    const bool reduced = status_stops_when_motion_is_reduced(true);
    if (!reduced) fprintf(stderr, "A running status still requested frames under reduced motion.\n");
    return composer && models && reduced ? 0 : 1;
}
