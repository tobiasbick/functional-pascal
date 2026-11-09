/* Small ABI adapters for options structs and compound-literal CUI macros. */
#include "fpas_cui.h"
#include "platform/cui_platform.h"
#include <math.h>
#include <stdlib.h>

/** Creates a native application with scalar options, copying its title.
 * @param title Borrowed UTF-8 title, valid for this call.
 * @param width Logical window width.
 * @param height Logical window height.
 * @param renderer A cui_renderer_kind value.
 * @param continuous Nonzero to request continuous frames.
 * @return The owned app, or NULL on creation failure. */
CUI_API cui_app *fpas_cui_window_create(const char *title, int width, int height,
                                      int renderer, int continuous) {
    const cui_app_desc options = {
        .title = title, .width = width, .height = height,
        .renderer = (cui_renderer_kind)renderer, .continuous = continuous != 0
    };
    return cui_app_create(&options);
}

void fpas_cui_finish_frame_for_destroy(cui_app *app) {
    if (!app->in_frame) return;
    cui_end_frame(app->ui);
    app->in_frame = false;
}

void fpas_cui_set_resize_renderer(cui_app *app, void (*render)(cui_ctx *, void *), void *user) {
    app->frame_cb = render;
    app->frame_user = render ? user : NULL;
}

/** Captures a view after at least one frame has been presented.
 * @param app Owned UI-thread application.
 * @param path Borrowed UTF-8 destination filename.
 * @return Nonzero on success; all temporary pixel memory is freed. */
CUI_API int fpas_cui_capture(cui_app *app, const char *path) {
    cui_ctx *ui = cui_app_ui(app);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    const cui_vec2 viewport = cui_viewport_size(ui);
    float scale = cui_dpi_scale(ui);
    double current_width = (double)(viewport.x * scale + 0.5f);
    double current_height = (double)(viewport.y * scale + 0.5f);
    if (!isfinite(current_width) || !isfinite(current_height) ||
        current_width < 1 || current_height < 1 ||
        current_width > INT32_MAX / 4 || current_height > INT32_MAX) return 0;
    int width = (int)current_width, height = (int)current_height;
    /* Screenshot validates the previous draw size before rendering the current
     * view. Reserve both sizes so a resize cannot underallocate its output. */
    if (draw->width_px > width) width = draw->width_px;
    if (draw->height_px > height) height = draw->height_px;
    if (width > INT32_MAX / 4 ||
        (size_t)height > SIZE_MAX / ((size_t)width * 4)) return 0;
    unsigned char *pixels = malloc((size_t)width * (size_t)height * 4);
    if (!pixels) return 0;
    bool ok = cui_app_screenshot(app, pixels, width, height);
    if (ok) {
        draw = cui_get_draw_data(ui);
        int stride = (cui_app_is_software(app) ? width : draw->width_px) * 4;
        ok = cui_write_png(path, pixels, draw->width_px, draw->height_px, stride);
    }
    free(pixels);
    return ok ? 1 : 0;
}
