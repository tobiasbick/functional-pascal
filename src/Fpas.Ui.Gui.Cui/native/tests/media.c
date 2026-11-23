// [Documentation](../../../../docs/ui/gui/social-and-media/README.md)
#include "../fpas_cui.h"
#include <stdio.h>

typedef struct { int created, destroyed, updates; bool valid_update; const uint8_t *expected; } image_probe;

static cui_texture upload(void *user, int width, int height, cui_texture_format format, const void *pixels) {
    image_probe *probe = user;
    if (format != CUI_TEXTURE_RGBA8 || width * height != 2 || !pixels) return 0;
    return (cui_texture)++probe->created;
}

static void update(void *user, cui_texture texture, int x, int y, int w, int h, const void *pixels, int stride) {
    image_probe *probe = user;
    ++probe->updates;
    probe->valid_update = texture == 1 && x == 1 && y == 0 && w == 1 && h == 1 &&
        stride == 8 && pixels == probe->expected;
}

static void release(void *user, cui_texture texture) {
    image_probe *probe = user;
    if (texture) ++probe->destroyed;
}

static cui_ctx *create_context(image_probe *probe) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .no_default_fonts = true,
        .renderer = { .user = probe, .texture_create = upload, .texture_update = update, .texture_destroy = release } });
    if (ui) cui_set_viewport(ui, 320, 240, 1);
    return ui;
}

static cui_id draw_image(cui_ctx *ui, fpas_media_source source) {
    cui_image_view view = {0};
    cui_begin_frame(ui, 1.0f / 60.0f);
    fpas_cui_image_viewer(ui, "image", &source, &view, 2, 200, 2, 160, 0, 3, 0);
    const cui_id key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    return key;
}

static cui_id draw_presence(cui_ctx *ui, const char *path, int maximum) {
    const fpas_presence_user users[] = {
        { .name = "Alice", .avatar = path }, { .name = "Bob", .avatar = path }
    };
    cui_begin_frame(ui, 1.0f / 60.0f);
    fpas_cui_presence(ui, "presence", users, 2, maximum, 24);
    const cui_id key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
    return key;
}

static bool presence_releases_avatars(void) {
    static const char path[] = "fpas-cui-test-presence.png";
    /* Two RGBA pixels, black and white. */
    static const unsigned char png[] = {
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0xf4, 0x22, 0x7f, 0x8a, 0x00, 0x00, 0x00,
        0x0e, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x60, 0x60, 0xf8,
        0x0f, 0x02, 0x00, 0x0e, 0xfa, 0x04, 0xfc, 0x45, 0xb0, 0x1d, 0x8b, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    };
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    FILE *fixture = fopen(path, "wbx");
    bool passed = false;
    if (fixture) {
        const size_t size = fwrite(png, 1, sizeof png, fixture);
        if (fclose(fixture) == 0 && size == sizeof png) {
            const cui_id key = draw_presence(ui, path, 2);
            passed = probe.created == 2 && probe.destroyed == 0;
            draw_presence(ui, path, 1);
            passed = passed && probe.created == 2 && probe.destroyed == 1;
            fpas_cui_forget(ui, key);
            passed = passed && probe.destroyed == 2;
        }
        passed = remove(path) == 0 && passed;
    }
    cui_destroy(ui);
    return passed;
}

int main(void) {
    static const uint8_t pixels[] = { 0, 0, 0, 255, 255, 255, 255, 255 };
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return 1;

    fpas_media_source source = { .pixels = pixels, .width = 2, .height = 1, .version = 1 };
    const cui_id key = draw_image(ui, source);
    draw_image(ui, source);
    bool passed = probe.created == 1 && probe.destroyed == 0;

    static const uint8_t patched[] = { 0, 0, 0, 255, 13, 42, 70, 255 };
    probe.expected = patched + 4;
    source.pixels = patched;
    source.previous = source.version++;
    source.x = 1;
    source.region_width = source.region_height = 1;
    draw_image(ui, source);
    draw_image(ui, source);
    passed = passed && probe.created == 1 && probe.destroyed == 0 && probe.updates == 1 && probe.valid_update;

    /* Revision 3 was never submitted to this slot. Upload the full revision 4. */
    source.previous = 3;
    source.version = 4;
    draw_image(ui, source);
    source.previous = 0;
    source.width = 1;
    source.height = 2;
    draw_image(ui, source);
    passed = passed && probe.created == 3 && probe.destroyed == 2;

    fpas_cui_forget(ui, key);
    passed = passed && probe.destroyed == 3;
    draw_image(ui, source);
    cui_destroy(ui);
    passed = passed && probe.created == 4 && probe.destroyed == 4;
    const bool avatars_released = presence_releases_avatars();
    if (!passed) fprintf(stderr, "Media upload, reuse, replacement or release did not match source lifetime.\n");
    if (!avatars_released) fprintf(stderr, "Presence did not release hidden or retired avatar textures.\n");
    return passed && avatars_released ? 0 : 1;
}
