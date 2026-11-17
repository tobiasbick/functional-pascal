#include "../../fpas_cui.h"

CUI_API float fpas_cui_canvas_font_scale(cui_ctx *ui) { return cui_theme_get(ui)->font_scale; }

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

CUI_API void fpas_cui_canvas_image(cui_ctx *ui, cui_id key, int index, const fpas_media_source *source,
    float x, float y, float w, float h, float radius, uint32_t tint) {
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
    if (texture) cui_draw_image(ui, texture, (cui_rect){x, y, w, h},
        (cui_rect){0, 0, 1, 1}, CUI_CORNERS(radius), fpas_chart_color(-1, tint));
}

CUI_API void fpas_cui_canvas_primitive(cui_ctx *ui, int kind, float x, float y, const float *v,
    uint32_t color, uint32_t second, const char *text) {
    const cui_color c = fpas_chart_color(-1, color), other = fpas_chart_color(-1, second);
    cui_vec2 p = {x + v[0], y + v[1]};
    if (kind == 0) cui_draw_rect(ui, (cui_rect){p.x,p.y,v[2],v[3]}, CUI_CORNERS(v[4]), c);
    else if (kind == 1) cui_draw_circle(ui, p, v[2], c);
    else if (kind == 2) cui_draw_line(ui, p, (cui_vec2){x+v[2],y+v[3]}, v[4], c);
    else if (kind == 3) cui_draw_rect_border(ui, (cui_rect){p.x,p.y,v[2],v[3]}, CUI_CORNERS(v[4]), c, other, v[5]);
    else if (kind == 4) cui_draw_rect_gradient(ui, (cui_rect){p.x,p.y,v[2],v[3]}, CUI_CORNERS(v[4]), c, other);
    else if (kind == 5) cui_draw_shadow(ui, (cui_rect){p.x,p.y,v[2],v[3]}, CUI_CORNERS(v[4]), v[5], v[6], (cui_vec2){v[7],v[8]}, c);
    else if (kind == 6) cui_draw_arc(ui, p, v[2], v[3], v[4], v[5], c);
    else if (kind == 7) cui_draw_text(ui, (int)v[2], v[3], p, c, text, -1);
    else if (kind == 8) cui_draw_icon(ui, text, (cui_rect){p.x,p.y,v[2],v[3]}, v[4], c);
    else if (kind == 9) cui_draw_emoji(ui, text, -1, (cui_rect){p.x,p.y,v[2],v[3]});
    else if (kind == 10) cui_push_clip(ui, (cui_rect){p.x,p.y,v[2],v[3]});
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
