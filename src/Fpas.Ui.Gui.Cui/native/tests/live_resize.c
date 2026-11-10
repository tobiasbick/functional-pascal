#include "../fpas_cui.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>

static void render_resize(cui_ctx *ui, void *user) {
    int *frames = user;
    ++*frames;
    cui_heading(ui, "Live resize", 2);
}

int main(void) {
    const cui_app_desc options = { .title = "CUI live resize", .renderer = CUI_RENDERER_SOFTWARE, .hidden = true };
    cui_app *app = cui_app_create(&options);
    if (!app) return 1;
    int frames = 0;
    HWND window = (HWND)cui_app_native_window(app);
    fpas_cui_set_resize_renderer(app, render_resize, &frames);
    SendMessageW(window, WM_ENTERSIZEMOVE, 0, 0);
    cui_app_set_size(app, 600, 400);
    int attached = frames;
    fpas_cui_set_resize_renderer(app, NULL, NULL);
    cui_app_set_size(app, 640, 480);
    SendMessageW(window, WM_EXITSIZEMOVE, 0, 0);
    cui_app_destroy(app);
    if (attached > 0 && frames == attached) return 0;
    fprintf(stderr, "Unexpected live-resize callback count: attached=%d, detached=%d\n", attached, frames - attached);
    return 1;
}
