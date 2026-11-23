// [Documentation](../../../../docs/ui/gui/native-build.md)
#include "../fpas_cui.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <GL/gl.h>
#include <stdio.h>

typedef int (WINAPI *get_interval_fn)(void);

/* Observe the driver's configured interval after a transition, allowing the
 * compositor to take multiple frames without depending on the monitor rate. */
static bool resume_frames(cui_app *app, get_interval_fn interval, bool no_vsync) {
    const DWORD start = GetTickCount();
    if (interval() != 0) return false;
    do {
        if (!cui_app_frame(app)) return false;
        cui_heading(cui_app_ui(app), "Window transition", 2);
        if (!fpas_cui_present(app)) return false;
        if (interval() != 0) return !no_vsync && interval() == 1;
    } while (GetTickCount() - start < (no_vsync ? 650u : 4000u));
    return no_vsync;
}

static int check_transition(bool no_vsync) {
    cui_app *app = cui_app_create(&(cui_app_desc){ .title = "Window transition", .continuous = true,
        .renderer = CUI_RENDERER_OPENGL, .no_vsync = no_vsync });
    if (!app) return 77;
    int status = 77;
    get_interval_fn interval = (get_interval_fn)(void *)wglGetProcAddress("wglGetSwapIntervalEXT");
    intptr_t address = (intptr_t)interval;
    if (cui_app_is_software(app) || address == -1 || (uintptr_t)address <= 3 ||
        interval() != (no_vsync ? 0 : 1)) goto done;

    status = 1;
    cui_app_maximize(app);
    if (!resume_frames(app, interval, no_vsync)) goto done;
    SendMessageW((HWND)cui_app_native_window(app), WM_SYSCOMMAND, SC_RESTORE, 0);
    if (!resume_frames(app, interval, no_vsync)) goto done;
    cui_app_minimize(app);
    if (interval() != 0) goto done;
    cui_app_restore(app);
    if (resume_frames(app, interval, no_vsync)) status = 0;
done:
    cui_app_destroy(app);
    return status;
}

int main(void) {
    int status = check_transition(false);
    if (status == 0) status = check_transition(true);
    if (status == 77) fprintf(stderr, "OpenGL with observable WGL swap control required.\n");
    if (status == 1) fprintf(stderr, "Unexpected WGL swap interval across a window transition.\n");
    return status;
}
