#include "../fpas_cui.h"
#include <stdio.h>

typedef struct { int created, destroyed; } image_probe;

static cui_texture upload(void *user, int width, int height, cui_texture_format format, const void *pixels) {
    image_probe *probe = user;
    if (format != CUI_TEXTURE_RGBA8 || width * height != 2 || !pixels) return 0;
    return (cui_texture)++probe->created;
}

static void release(void *user, cui_texture texture) {
    image_probe *probe = user;
    if (texture) ++probe->destroyed;
}

static cui_id draw_image(cui_ctx *ui, fpas_media_source source) {
    cui_image_view view = {0};
    cui_begin_frame(ui, 1.0f / 60.0f);
    fpas_cui_image_viewer(ui, "image", &source, &view, 2, 200, 2, 160, 0, 3, 0);
    const cui_id key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    return key;
}

int main(void) {
    static const uint8_t pixels[] = { 0, 0, 0, 255, 255, 255, 255, 255 };
    image_probe probe = {0};
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .no_default_fonts = true,
        .renderer = { .user = &probe, .texture_create = upload, .texture_destroy = release } });
    if (!ui) return 1;
    cui_set_viewport(ui, 320, 240, 1);

    fpas_media_source source = { .pixels = pixels, .width = 2, .height = 1, .version = 1 };
    const cui_id key = draw_image(ui, source);
    draw_image(ui, source);
    bool passed = probe.created == 1 && probe.destroyed == 0;

    source.version++;
    draw_image(ui, source);
    source.width = 1;
    source.height = 2;
    draw_image(ui, source);
    passed = passed && probe.created == 3 && probe.destroyed == 2;

    fpas_cui_forget(ui, key);
    passed = passed && probe.destroyed == 3;
    draw_image(ui, source);
    cui_destroy(ui);
    passed = passed && probe.created == 4 && probe.destroyed == 4;
    if (!passed) fprintf(stderr, "Media upload, reuse, replacement or release did not match source lifetime.\n");
    return passed ? 0 : 1;
}
