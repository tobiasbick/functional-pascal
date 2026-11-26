// [Documentation](../../../../docs/ui/gui/cui.md#window-ownership-and-frames)
/* Small ABI adapters for options structs and compound-literal CUI macros. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdlib.h>

/** Creates a native application with scalar options, copying its title.
 * @param title Borrowed UTF-8 title, valid for this call.
 * @param width Logical window width, also the normal width of a placement.
 * @param height Logical window height, also the normal height of a placement.
 * @param renderer A cui_renderer_kind value.
 * @param continuous Nonzero to request continuous frames.
 * @param placed Nonzero to open at the following placement.
 * @param x Left edge of the content in desktop coordinates.
 * @param y Top edge of the content in desktop coordinates.
 * @param has_position Nonzero when x and y name a saved place.
 * @param maximized Nonzero to open maximized.
 * @param minimum_width Minimum logical width, zero for CUI's default.
 * @param minimum_height Minimum logical height, zero for CUI's default.
 * @param fixed_size Nonzero to create a nonresizable window.
 * @param app_id Optional Linux desktop application identity, copied at creation.
 * @param no_vsync Nonzero to present without waiting for OpenGL vertical sync.
 * @param hidden Nonzero to create the window without showing it.
 * @param reading_direction Native reading direction inherited by the window's widgets.
 * @return The owned app, or NULL on creation failure. */
CUI_API cui_app *fpas_cui_window_create(const char *title, int width, int height,
                                      int renderer, int continuous, int placed,
                                      int x, int y, int has_position, int maximized,
                                      int minimum_width, int minimum_height, int fixed_size,
                                      const char *app_id, int no_vsync, int hidden, int reading_direction) {
    const cui_window_placement placement = {
        .x = x, .y = y, .width = width, .height = height,
        .state = maximized ? CUI_WINDOW_MAXIMIZED : CUI_WINDOW_NORMAL, .has_position = has_position != 0
    };
    const cui_app_desc options = {
        .title = title, .width = width, .height = height,
        .renderer = (cui_renderer_kind)renderer, .continuous = continuous != 0,
        .no_vsync = no_vsync != 0, .hidden = hidden != 0,
        .min_width = minimum_width, .min_height = minimum_height,
        .fixed_size = fixed_size != 0, .app_id = app_id,
        .placement = placed ? &placement : NULL
    };
    cui_app *app = cui_app_create(&options);
    if (app) cui_app_ui(app)->direction = (cui_direction)reading_direction;
#ifdef _WIN32
    if (app) {
        /* The FPAS Windows executable stores ApplicationIcon under IDI_APPLICATION. */
        HICON icon = LoadIconW(GetModuleHandleW(NULL), (LPCWSTR)IDI_APPLICATION);
        if (icon) {
            HWND window = (HWND)cui_app_native_window(app);
            SendMessageW(window, WM_SETICON, ICON_SMALL, (LPARAM)icon);
            SendMessageW(window, WM_SETICON, ICON_BIG, (LPARAM)icon);
        }
    }
#endif
    return app;
}

void fpas_cui_finish_frame_for_destroy(cui_app *app) {
    if (!app->in_frame) return;
    cui_end_frame(app->ui);
    app->in_frame = false;
}

int fpas_cui_present(cui_app *app) {
    cui__app_render(app);
    const cui_draw_data *draw = cui_get_draw_data(app->ui);
    return draw->width_px > 0 && draw->height_px > 0 &&
        (app->gl != NULL || app->sw_pixels != NULL);
}

int fpas_cui_frame(cui_app *app, int defer_close) {
    if (cui_app_frame(app)) return 1;
    if (!defer_close) return 0;
    app->running = true;
    cui__app_begin(app);
    return 2;
}

int fpas_cui_last_node(cui_ctx *ui) { return ui->last_node; }

cui_rect fpas_cui_node_rect(cui_ctx *ui, int node) {
    if (node < 0 || node >= ui->node_count) return (cui_rect){0};
    cui_rect rect = ui->nodes[node].rect;
    /* Apply the visual transform from each ancestor, as CUI does for hit testing. */
    for (int index = node; index >= 0; index = ui->nodes[index].parent) {
        const cui_node *part = &ui->nodes[index];
        if (part->scale == 1.0f && part->tx == 0 && part->ty == 0) continue;
        const float x = part->rect.x + part->rect.w * 0.5f;
        const float y = part->rect.y + part->rect.h * 0.5f;
        rect.x = x + (rect.x - x) * part->scale + part->tx;
        rect.y = y + (rect.y - y) * part->scale + part->ty;
        rect.w *= part->scale;
        rect.h *= part->scale;
    }
    return rect;
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
