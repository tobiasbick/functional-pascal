// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/painting.md)
#include "../../fpas_cui.h"

CUI_API float fpas_cui_canvas_font_scale(cui_ctx *ui) { return cui_theme_get(ui)->font_scale; }

/** Resets a spring when its value cannot be represented or its duration is below the frame resolution. */
static float fpas_canvas_snap(cui_ctx *ui, cui_id key, float target) {
    const float motion = ui->theme.motion;
    ui->theme.motion = 0.0f;
    const float value = cui_animate(ui, key, target);
    ui->theme.motion = motion;
    return value;
}

/** Steps a canvas-local spring with native redraw scheduling and reduced motion. */
CUI_API float fpas_cui_canvas_animate(cui_ctx *ui, cui_id canvas, const char *id,
    float target, float duration, float bounce) {
    const cui_id key = cui__hash_str(id, canvas);
    const bool pending = ui->animating;
    float value = target;
    if (duration == 0) {
        value = cui_animate(ui, key, target);
    } else {
        const float step = ui->step;
        const float motion = ui->theme.motion;
        const float elapsed = fminf(step, 0.1f);
        const float slice = (duration * motion) / 8.0f;
        if (motion > 0.0f && slice < 1.0f / 240.0f) {
            /* Keep the native integrator stable for springs shorter than its normal time step. */
            if (slice < 0.00001f || elapsed > 256.0f * slice)
                return fpas_canvas_snap(ui, key, target);
            const int count = CUI_MAX(1, (int)ceilf(elapsed / slice));
            ui->step = elapsed / (float)count;
            for (int i = 0; i < count; i++) {
                ui->animating = pending;
                value = cui_animate_spring(ui, key, target, duration, bounce);
                if (!isfinite(value)) break;
            }
            ui->step = step;
        } else {
            value = cui_animate_spring(ui, key, target, duration, bounce);
        }
    }
    if (isfinite(value)) return value;
    ui->animating = pending;
    return fpas_canvas_snap(ui, key, target);
}

/** Resolves a color token against the native palette at paint time. */
uint32_t fpas_cui_canvas_color(cui_ctx *ui, int token) {
    const cui_color c = cui_token_color(ui, (cui_token)token);
    return (uint32_t)c.r | ((uint32_t)c.g << 8) | ((uint32_t)c.b << 16) | ((uint32_t)c.a << 24);
}

typedef struct { fpas_media_image *items; int count, capacity; } fpas_canvas_images;

static void fpas_canvas_images_free(cui_ctx *ui, void *data) {
    fpas_canvas_images *images = data;
    for (int i = 0; i < images->count; i++) fpas_media_release(ui, &images->items[i]);
    cui__free(ui, images->items);
}

CUI_API void fpas_cui_canvas_images_finish(cui_ctx *ui, cui_id key, int count) {
    fpas_canvas_images *images = cui_state(ui, key, sizeof *images);
    while (images->count > count) {
        fpas_media_image *image = &images->items[--images->count];
        fpas_media_release(ui, image);
        *image = (fpas_media_image){0};
    }
}

/** Resolves a cached image source and draws the requested texture region. */
CUI_API void fpas_cui_canvas_image(cui_ctx *ui, cui_id key, int index, const fpas_media_source *source,
    float x, float y, float w, float h, cui_corners radii, cui_rect uv, uint32_t tint) {
    fpas_canvas_images *images = cui_state(ui, key, sizeof *images);
    cui_state_on_free(ui, key, fpas_canvas_images_free);
    if (index >= images->count) {
        if (images->count == images->capacity) {
            const int old = images->capacity;
            images->capacity = old ? old * 2 : 4;
            images->items = cui__realloc(ui, images->items, (size_t)images->capacity * sizeof *images->items);
            memset(images->items + old, 0, (size_t)(images->capacity - old) * sizeof *images->items);
        }
        images->count++;
    }
    cui_texture texture = fpas_media_update(ui, &images->items[index], source)->texture;
    if (texture) cui_draw_image(ui, texture, (cui_rect){x, y, w, h}, uv, radii, fpas_chart_color(-1, tint));
}

/** Draws one native primitive in canvas-local coordinates. */
CUI_API void fpas_cui_canvas_primitive(cui_ctx *ui, int kind, float x, float y, const float *v,
    uint32_t color, uint32_t second, const char *text, cui_corners radii) {
    const cui_color c = fpas_chart_color(-1, color), other = fpas_chart_color(-1, second);
    cui_vec2 p = {x + v[0], y + v[1]};
    if (kind == 0) cui_draw_rect(ui, (cui_rect){p.x,p.y,v[2],v[3]}, radii, c);
    else if (kind == 1) cui_draw_circle(ui, p, v[2], c);
    else if (kind == 2) cui_draw_line(ui, p, (cui_vec2){x+v[2],y+v[3]}, v[4], c);
    else if (kind == 3) cui_draw_rect_border(ui, (cui_rect){p.x,p.y,v[2],v[3]}, radii, c, other, v[4]);
    else if (kind == 4) cui_draw_rect_gradient(ui, (cui_rect){p.x,p.y,v[2],v[3]}, radii, c, other);
    else if (kind == 5) cui_draw_shadow(ui, (cui_rect){p.x,p.y,v[2],v[3]}, radii, v[4], v[5], (cui_vec2){v[6],v[7]}, c);
    else if (kind == 6) cui_draw_arc(ui, p, v[2], v[3], v[4], v[5], c);
    else if (kind == 7) cui_draw_text(ui, (int)v[2], v[3], p, c, text, -1);
    else if (kind == 8) cui_draw_icon(ui, text, (cui_rect){p.x,p.y,v[2],v[3]}, v[4], c);
    else if (kind == 9) cui_draw_emoji(ui, text, -1, (cui_rect){p.x,p.y,v[2],v[3]});
}

/** CUI-176 (docs/future/cui-upstream-bugs.md#cui-176): count only accepted clips during unwind. */
int fpas_cui_canvas_push_clip(cui_ctx *ui, float x, float y, float width, float height) {
    const int depth = ui->dl.clip_depth;
    cui_push_clip(ui, (cui_rect){x, y, width, height});
    return ui->dl.clip_depth > depth;
}

CUI_API void fpas_cui_canvas_points(cui_ctx *ui, float x, float y, const cui_vec2 *p, int count,
    float thickness, uint32_t color, int kind) {
    cui_vec2 *points = cui__frame_alloc(ui, (size_t)count * sizeof *points);
    for (int i = 0; i < count; i++) points[i] = (cui_vec2){x + p[i].x, y + p[i].y};
    if (kind == 2) cui_draw_polygon(ui, points, count, fpas_chart_color(-1, color));
    else cui_draw_polyline(ui, points, count, thickness, fpas_chart_color(-1, color), kind != 0);
}

CUI_API cui_text_metrics fpas_cui_canvas_measure(cui_ctx *ui, int font, float size, const char *text) {
    return cui_measure_text(ui, font, size, text, -1);
}
