// [Documentation](../../../../docs/ui/gui/developer-tools/README.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

enum { PIPELINE, PROBLEMS, KEYMAP };

static const char *const stage_names[] = {"Build", "Test", "Package", "Deploy"};
static const cui_pipeline_job jobs[] = {
    {"One", 0, CUI_JOB_PASSED, 1}, {"Two", 1, CUI_JOB_FAILED, 1, true},
    {"Three", 2, CUI_JOB_PENDING, -1}, {"Four", 3, CUI_JOB_SKIPPED, -1}
};
static const cui_diagnostic problem = {
    CUI_SEVERITY_WARNING, "Check this line", "src/example.fpas", 2, 1, "fpas", "FP1001"
};
static const cui_keymap_binding bindings[] = {
    {{0, CUI_KEY_F2}, "Rename", 0}, {{0, CUI_KEY_F2}, "Inspect", 0},
    {{CUI_MOD_PRIMARY, CUI_KEY_S}, "Save", 1}
};
static const char *const categories[] = {"File", "Edit"};

static cui_ctx *create(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 1000, 700, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static fpas_dev_result draw(cui_ctx *ui, int kind, int flags, bool simple) {
    fpas_dev_result result;
    if (kind == PIPELINE)
        fpas_cui_pipeline(ui, "pipeline", stage_names, 4, jobs, 4, -2, 1, 0, flags, &result);
    else if (kind == PROBLEMS)
        fpas_cui_problems(ui, "problems", &problem, 1, simple ? -1 : 180, simple, 1, 0, flags, &result);
    else
        fpas_cui_keymap(ui, "keymap", bindings, 3, categories, 2, simple ? -1 : 0, simple, 1, 0, flags, &result);
    return result;
}

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static bool resize(int kind) {
    cui_ctx *ui = create();
    int wide = 0, narrow = 0;
    bool ok = true;
    for (int frame = 0; frame < 8; ++frame) {
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width=CUI_PX(frame >= 3 && frame <= 5 ? 440 : 900)) draw(ui, kind, 1, false);
        cui_end_frame(ui);
        const int measure = kind == PIPELINE ? (int)cui_get_rect(ui, cui_make_id(ui, "pipeline")).h
            : cui_get_draw_data(ui)->vertex_count;
        if (frame == 2) wide = measure;
        if (frame == 5) narrow = measure;
        if (frame >= 2) ok &= cui_needs_redraw(ui) == (frame == 3 || frame == 6);
        if (frame == 3 || frame == 7) ok &= measure == wide;
    }
    cui_destroy(ui);
    if (!ok || !wide || !narrow || wide == narrow) fprintf(stderr, "Inspector %d did not reconcile its width\n", kind);
    return ok && wide && narrow && wide != narrow;
}

static bool keyboard(int kind) {
    cui_ctx *ui = create();
    cui_rect anchor = {0};
    bool ok = true;
    for (int frame = 0; frame < (kind == KEYMAP ? 9 : 8); ++frame) {
        if (kind == PROBLEMS && (frame == 2 || frame == 3)) key(ui, CUI_KEY_DOWN);
        if (frame == 4) {
            cui_input_mouse_move(ui, anchor.x + 5, anchor.y + anchor.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_RIGHT, true);
        }
        if (frame == 5) cui_input_mouse_button(ui, CUI_MOUSE_RIGHT, false);
        if (frame == 5 || frame == 7) key(ui, kind == KEYMAP ? CUI_KEY_RIGHT : CUI_KEY_ENTER);
        if (frame == 6) key(ui, CUI_KEY_ESCAPE);
        cui_begin_frame(ui, .04f);
        fpas_dev_result result;
        cui_column(ui, .height=CUI_PX(400)) { result = draw(ui, kind, frame == 0 ? 3 : 1, true); }
        const bool focused = fpas_cui_focused(ui, fpas_cui_last_node(ui));
        cui_button(ui, "Open context here");
        const cui_id button = cui_last_id(ui);
        bool open = false;
        cui_context_menu(ui, "cover") { open = true; cui_menu_item(ui, "Menu command"); }
        cui_end_frame(ui);
        anchor = cui_get_rect(ui, button);
        if (frame == 0) ok &= focused;
        if (frame == 4 || frame == 5) ok &= open;
        if (kind == KEYMAP) ok &= result.index == (frame >= 7 ? CUI_MOD_PRIMARY : 0) && result.count == 1 &&
            result.flags == (frame == 7);
        else ok &= result.action == (frame == 7 ? 0 : -1);
        if (kind == PIPELINE) ok &= result.index == -1;
        if (kind == PROBLEMS && frame >= 3) ok &= result.index == 0 && result.count == 1;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Inspector %d focus, activation or popup ownership differs\n", kind);
    return ok;
}

static bool disabled_problems_wheel(void) {
    cui_ctx *idle = create(), *wheel = create();
    cui_diagnostic diagnostics[30];
    for (int i = 0; i < 30; ++i) {
        diagnostics[i] = problem;
        diagnostics[i].line = i + 1;
    }
    for (int frame = 0; frame < 7; ++frame) {
        if (frame >= 2) {
            cui_input_mouse_move(idle, 30, 120);
            cui_input_mouse_move(wheel, 30, 120);
        }
        if (frame == 3 || frame == 5) cui_input_scroll(wheel, 0, -4);
        fpas_dev_result result;
        cui_begin_frame(idle, .04f); cui_begin_frame(wheel, .04f);
        fpas_cui_problems(idle, "problems", diagnostics, 30, 200, 1, 1, 0, frame < 3, &result);
        fpas_cui_problems(wheel, "problems", diagnostics, 30, 200, 1, 1, 0, frame < 3, &result);
        cui_end_frame(idle); cui_end_frame(wheel);
    }
    const cui_draw_data *a = cui_get_draw_data(idle), *b = cui_get_draw_data(wheel);
    const bool ok = a->vertex_count == b->vertex_count &&
        memcmp(a->vertices, b->vertices, (size_t)a->vertex_count * sizeof *a->vertices) == 0;
    cui_destroy(idle); cui_destroy(wheel);
    if (!ok) fprintf(stderr, "Disabled Problems list scrolled\n");
    return ok;
}

int main(void) {
    bool ok = disabled_problems_wheel();
    for (int i = PIPELINE; i <= KEYMAP; ++i) {
        ok &= resize(i);
        ok &= keyboard(i);
    }
    return ok ? 0 : 1;
}
