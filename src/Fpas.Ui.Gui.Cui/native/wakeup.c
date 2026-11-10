#include "fpas_cui.h"

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

void *fpas_cui_wake_open(cui_app *app) {
    return cui_app_native_window(app);
}

int fpas_cui_wake_post(void *target) {
    return PostMessageW((HWND)target, WM_NULL, 0, 0) != 0;
}

void fpas_cui_wake_close(void *target) {
    (void)target;
}

#else
#include <X11/Xlib.h>
#include <stdlib.h>

typedef struct wake_target {
    Display *display;
    Window window;
    Atom message;
} wake_target;

void *fpas_cui_wake_open(cui_app *app) {
    wake_target *target = malloc(sizeof *target);
    if (!target) return NULL;
    /* Use a separate connection so workers never touch the UI thread's event queue. */
    target->display = XOpenDisplay(NULL);
    if (!target->display) { free(target); return NULL; }
    target->window = (Window)(uintptr_t)cui_app_native_window(app);
    target->message = XInternAtom(target->display, "_FPAS_CUI_WAKE", False);
    return target;
}

int fpas_cui_wake_post(void *handle) {
    wake_target *target = handle;
    XEvent event = { .xclient = {
        .type = ClientMessage, .display = target->display, .window = target->window,
        .message_type = target->message, .format = 32
    } };
    int sent = XSendEvent(target->display, target->window, False, NoEventMask, &event);
    XFlush(target->display);
    return sent != 0;
}

void fpas_cui_wake_close(void *handle) {
    wake_target *target = handle;
    XCloseDisplay(target->display);
    free(target);
}
#endif
