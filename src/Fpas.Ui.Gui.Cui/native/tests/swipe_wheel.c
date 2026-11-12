#include "../fpas_cui.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>

static bool frame(cui_ctx *ui, float elapsed) {
    const char *labels[] = { "Done" };
    const char *icons[] = { CUI_ICON_CHECK };
    const int tones[] = { CUI_TONE_SUCCESS };
    int nodes[2];
    cui_begin_frame(ui, elapsed);
    const int viewport = fpas_cui_viewport(ui, "page", 1, 1);
    const int row = fpas_cui_swipe_begin(ui, "task", labels, icons, tones, 1, 0, 1, NULL, nodes);
    cui_text(ui, "Review the design tokens", .width = CUI_FILL);
    fpas_cui_swipe_end(ui, row, NULL);
    cui_box(ui, .height = CUI_PX(600)) { }
    cui_scroll_end(ui);
    cui_end_frame(ui);
    float offset, maximum;
    fpas_cui_scroll_metrics(ui, viewport, &offset, &maximum);
    return nodes[1] >= 0 && offset == 0 && maximum > 0;
}

int main(void) {
    const cui_app_desc options = {
        .title = "CUI horizontal wheel", .width = 600, .height = 400,
        .renderer = CUI_RENDERER_SOFTWARE, .hidden = true
    };
    cui_app *app = cui_app_create(&options);
    if (!app) return 1;
    cui_ctx *ui = cui_app_ui(app);
    HWND window = (HWND)cui_app_native_window(app);
    cui_set_viewport(ui, 600, 400, 1);
    cui_set_reduced_motion(ui, true);
    frame(ui, 0.016f);
    const bool initially_closed = !frame(ui, 0.016f);
    SendMessageW(window, WM_MOUSEMOVE, 0, MAKELPARAM(200, 20));
    /* Windows delivers the touchpad's horizontal scroll in wheel units. */
    SendMessageW(window, WM_MOUSEHWHEEL, MAKEWPARAM(0, (WORD)-360), 0);
    frame(ui, 0.016f);
    const bool revealed = frame(ui, 0.2f);
    cui_app_destroy(app);
    if (initially_closed && revealed) return 0;
    fprintf(stderr, "WM_MOUSEHWHEEL did not reveal the swipe action without moving the scroll view.\n");
    return 1;
}
