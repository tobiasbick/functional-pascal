// [Documentation](../../../../docs/ui/gui/social-and-media/README.md)
// [Avatar upload](../../../../docs/ui/gui/capture/avatar-upload.md)
// [Image viewer](../../../../docs/ui/gui/social-and-media/image-viewer.md)
// [Image cropper](../../../../docs/ui/gui/social-and-media/image-cropper.md)
// [Image compare](../../../../docs/ui/gui/social-and-media/image-compare.md)
// [Now playing](../../../../docs/ui/gui/motion/now-playing.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

static bool presence_releases_avatars(const char *path) {
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    const cui_id key = draw_presence(ui, path, 2);
    bool passed = probe.created == 2 && probe.destroyed == 0;
    draw_presence(ui, path, 1);
    passed = passed && probe.created == 2 && probe.destroyed == 1;
    fpas_cui_forget(ui, key);
    passed = passed && probe.destroyed == 2;
    cui_destroy(ui);
    return passed;
}

static int draw_avatar_upload(cui_ctx *ui, const char *path, float size, bool enabled,
    cui_key key, const char *drop, int *has_image, bool *reported_drop) {
    const cui_rect rect = drop ? cui_last_rect(ui) : (cui_rect){0};
    cui_begin_frame(ui, 1.0f / 60.0f);
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    if (drop) cui_input_file_drop(ui, &drop, 1, rect.x + rect.w / 2, rect.y + rect.h / 2);
    int node;
    const int action = fpas_cui_avatar_upload(ui, "avatar", "Maya Okafor", path, size,
        enabled, enabled, &node, has_image);
    const char *dropped = action == 1 ? cui_dropped_file(ui, 0) : NULL;
    *reported_drop = drop && dropped && strcmp(drop, dropped) == 0;
    cui_end_frame(ui);
    return action;
}

static bool avatar_upload_tracks_image(const char *path) {
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    cui_set_reduced_motion(ui, true);
    int has_image;
    bool reported_drop;
    int action = draw_avatar_upload(ui, NULL, -1, true, CUI_KEY_NONE, NULL, &has_image, &reported_drop);
    const cui_id key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_rect rect = cui_last_rect(ui);
    bool passed = action == 0 && !has_image && rect.w == 88 && rect.h == 88;

    action = draw_avatar_upload(ui, NULL, 64, true, CUI_KEY_SPACE, NULL, &has_image, &reported_drop);
    rect = cui_last_rect(ui);
    passed = passed && action == -1 && !has_image && rect.w == 64 && rect.h == 64;
    action = draw_avatar_upload(ui, NULL, 64, false, CUI_KEY_ENTER, path, &has_image, &reported_drop);
    passed = passed && action == 0 && !has_image && !reported_drop && probe.created == 0;

    /* A decoded drop takes precedence over a browse activation in the same frame. */
    action = draw_avatar_upload(ui, NULL, 64, true, CUI_KEY_ENTER, path, &has_image, &reported_drop);
    passed = passed && action == 1 && has_image && reported_drop && probe.created == 1;
    action = draw_avatar_upload(ui, path, 64, true, CUI_KEY_NONE, NULL, &has_image, &reported_drop);
    passed = passed && action == 0 && has_image && probe.created == 1 && probe.destroyed == 0;

    action = draw_avatar_upload(ui, path, 64, true, CUI_KEY_NONE, path, &has_image, &reported_drop);
    passed = passed && action == 1 && has_image && reported_drop && probe.created == 2 && probe.destroyed == 1;
    fpas_cui_forget(ui, key);
    passed = passed && probe.destroyed == 2;
    cui_destroy(ui);
    return passed && probe.destroyed == 2;
}

static bool test_avatar_textures(void) {
    static const char path[] = "fpas-cui-test-avatars.png";
    /* Two RGBA pixels, black and white. */
    static const unsigned char png[] = {
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0xf4, 0x22, 0x7f, 0x8a, 0x00, 0x00, 0x00,
        0x0e, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x60, 0x60, 0xf8,
        0x0f, 0x02, 0x00, 0x0e, 0xfa, 0x04, 0xfc, 0x45, 0xb0, 0x1d, 0x8b, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    };
    FILE *fixture = fopen(path, "wbx");
    if (!fixture) return false;
    const size_t size = fwrite(png, 1, sizeof png, fixture);
    const bool written = fclose(fixture) == 0 && size == sizeof png;
    const bool presence = written && presence_releases_avatars(path);
    const bool avatar = written && avatar_upload_tracks_image(path);
    if (!presence) {
        fprintf(stderr, "Presence did not release hidden or retired avatar textures.\n");
    }
    if (!avatar) {
        fprintf(stderr, "Avatar upload did not preserve drop priority, browse, bounds or texture lifetime.\n");
    }
    return remove(path) == 0 && presence && avatar;
}

/** Compares actual adapter and native drawing with both default and custom options. */
static bool photo_pixels_match(int widget, bool custom, bool dark) {
    enum { WIDTH = 420, HEIGHT = 382, PICTURE_WIDTH = 8, PICTURE_HEIGHT = 6 };
    uint8_t picture[PICTURE_WIDTH * PICTURE_HEIGHT * 4];
    for (int i = 0; i < PICTURE_WIDTH * PICTURE_HEIGHT; ++i) {
        picture[4 * i] = (uint8_t)(i * 9);
        picture[4 * i + 1] = 60;
        picture[4 * i + 2] = (uint8_t)(i * 17);
        picture[4 * i + 3] = i % 4 ? 255 : 0;
    }
    const fpas_media_source image = { .pixels = picture, .width = PICTURE_WIDTH,
        .height = PICTURE_HEIGHT, .version = 1 };
    const size_t bytes = WIDTH * HEIGHT * 4;
    uint8_t *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    bool valid = pixels[0] && pixels[1];
    const cui_size width = custom ? CUI_PX(270) : CUI_FIT;
    const cui_size height = custom ? CUI_PX(180) : CUI_FIT;
    const float maximum = custom ? 30 : 0;
    for (int pass = 0; pass < 2 && valid; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = renderer ? cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)}) : NULL;
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        const cui_texture native = pass ? 0 : cui_image_create(ui, image.width, image.height, picture);
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .02f);
            if (widget == 0) {
                cui_image_view view = { .zoom = custom ? 30 : 0, .cx = 4, .cy = 3 };
                if (pass) fpas_cui_image_viewer(ui, "image", &image, &view, custom ? 2 : 0, width.value,
                    custom ? 2 : 0, height.value, maximum, custom ? 7 : 1, 0);
                else cui_image_viewer(ui, "image", native, image.width, image.height, &view,
                    .width = width, .height = height, .max_zoom = maximum,
                    .no_toolbar = custom, .checkerboard = custom);
            } else if (widget == 1) {
                cui_rect crop = { 1, 1, 4, 4 };
                if (pass) fpas_cui_image_cropper(ui, "image", &image, &crop, custom ? 2 : 0, width.value,
                    custom ? 2 : 0, height.value, 0, 1, custom ? 3 : 1, 0);
                else cui_cropper(ui, "image", native, image.width, image.height, &crop,
                    .width = width, .height = height, .min_size = 1, .round = custom);
            } else {
                float position = .45f;
                const float aspect = custom ? (float)image.width / image.height : 0;
                if (pass) fpas_cui_image_compare(ui, "image", &image, &image, &position, custom ? 2 : 0,
                    width.value, height.value, aspect, custom ? "Original" : "Before",
                    custom ? "Current" : "After", 1, 0);
                else cui_image_compare(ui, "image", native, native, &position,
                    .width = width, .height = height.value, .aspect = aspect,
                    .before_label = custom ? "Original" : NULL, .after_label = custom ? "Current" : NULL);
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        if (native) cui_image_destroy(ui, native);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]);
    free(pixels[1]);
    if (!valid) fprintf(stderr, "Photo widget pixels differ: widget=%d custom=%d dark=%d.\n", widget, custom, dark);
    return valid;
}

/** Draws a scroll area with a viewer and returns the lower marker's current position. */
static float viewer_scroll_marker(cui_ctx *ui, const fpas_media_source *image, bool enabled) {
    cui_image_view view = {0};
    cui_begin_frame(ui, .02f);
    cui_scroll(ui, .id = "page", .height = CUI_PX(160)) {
        fpas_cui_image_viewer(ui, "viewer", image, &view, 1, 0, 2, 110, 0, enabled ? 3 : 2, 0);
        cui_column(ui, .height = CUI_PX(500)) { cui_button(ui, "Below"); }
    }
    cui_end_frame(ui);
    return cui_last_rect(ui).y;
}

/** Verifies only an enabled viewer consumes wheel input inside the scroll area. */
static bool viewer_routes_wheel(const fpas_media_source *image) {
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    cui_set_reduced_motion(ui, true);
    viewer_scroll_marker(ui, image, true);
    const float start = viewer_scroll_marker(ui, image, true);
    cui_input_mouse_move(ui, 100, 70);
    cui_input_scroll(ui, 0, -3);
    const float enabled = viewer_scroll_marker(ui, image, true);
    viewer_scroll_marker(ui, image, false);
    cui_input_scroll(ui, 0, -3);
    const float disabled = viewer_scroll_marker(ui, image, false);
    cui_destroy(ui);
    return start == enabled && disabled < enabled;
}

/** Draws the crop rectangle with a fixed 2:1 aspect and captures its proposal. */
static int draw_crop(cui_ctx *ui, const fpas_media_source *image, cui_rect *crop) {
    cui_begin_frame(ui, .02f);
    const int changed = fpas_cui_image_cropper(ui, "crop", image, crop, 2, 320, 2, 260, 2, 16, 1, 0);
    cui_end_frame(ui);
    return changed;
}

/** Verifies the initial model proposal and an aspect-preserving pointer resize. */
static bool crop_tracks_minimum(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = renderer ? cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)}) : NULL;
    if (!ui) { cui_sw_destroy(renderer); return false; }
    const size_t bytes = 640 * 400 * 4;
    uint8_t *picture = calloc(1, bytes);
    if (!picture) { cui_destroy(ui); cui_sw_destroy(renderer); return false; }
    const fpas_media_source source = { .pixels = picture, .width = 640, .height = 400, .version = 1 };
    cui_set_viewport(ui, 320, 260, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect crop = {0};
    bool valid = draw_crop(ui, &source, &crop) && crop.w > 0 && crop.w == crop.h * 2;
    valid = !draw_crop(ui, &source, &crop) && valid;

    crop = (cui_rect){ 20, 40, 120, 60 };
    draw_crop(ui, &source, &crop);
    const cui_rect bounds = cui_last_rect(ui);
    const float z = (bounds.w - 28) / 640;
    const float y = bounds.y + (bounds.h - 400 * z) / 2 + (crop.y + crop.h / 2) * z;
    cui_input_mouse_move(ui, bounds.x + 14 + (crop.x + crop.w) * z, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_crop(ui, &source, &crop);
    cui_input_mouse_move(ui, bounds.x + 14 + 40 * z, y);
    valid = draw_crop(ui, &source, &crop) && crop.w == 32 && crop.h == 16 && fabsf(crop.x - 20) < .01f && valid;

    free(picture);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    return valid;
}

/** Gives one ID two independent comparison states under distinct parent scopes. */
static bool comparison_owns_scoped_textures(const fpas_media_source *image) {
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    cui_id keys[2];
    for (int frame = 0; frame < 2; ++frame) {
        cui_begin_frame(ui, .02f);
        for (int i = 0; i < 2; ++i) {
            cui_push_id_int(ui, i);
            float position = .45f;
            fpas_cui_image_compare(ui, "compare", image, image, &position, 2, 200, 110, 2, "Before", "After", 1, 0);
            keys[i] = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
            cui_pop_id(ui);
        }
        cui_end_frame(ui);
    }
    bool valid = keys[0] != keys[1] && probe.created == 4 && probe.destroyed == 0;
    fpas_cui_forget(ui, keys[0]);
    valid = valid && probe.destroyed == 2;
    cui_destroy(ui);
    return valid && probe.destroyed == 4;
}

static bool player_releases_artwork(fpas_media_source source) {
    image_probe probe = {0};
    cui_ctx *ui = create_context(&probe);
    if (!ui) return false;
    const cui_id key = cui_make_id(ui, "player");
    const int uploads[] = { 1, 1, 1, 2 }, releases[] = { 0, 0, 1, 1 };
    bool valid = true;
    for (int frame = 0; frame < 4; ++frame) {
        float position = 0;
        int playing = 0, parts[4];
        cui_begin_frame(ui, .016f);
        fpas_cui_now_playing(ui, "player", "Track", "", frame == 2 ? NULL : &source, 0, 0,
            &position, 10, &playing, 2, 300, 3, -1, parts);
        cui_end_frame(ui);
        valid = valid && probe.created == uploads[frame] && probe.destroyed == releases[frame] &&
            parts[0] >= 0 && parts[1] < 0 && parts[2] >= 0 && parts[3] >= 0;
    }
    fpas_cui_forget(ui, key);
    valid = valid && probe.destroyed == 2;
    cui_destroy(ui);
    return valid && probe.destroyed == 2;
}

/** Verifies media interactions, texture ownership and strict native pixels. */
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
    const bool avatars_released = test_avatar_textures();
    const fpas_media_source two_pixels = { .pixels = pixels, .width = 2, .height = 1, .version = 1 };
    const bool wheel = viewer_routes_wheel(&two_pixels);
    const bool crop = crop_tracks_minimum();
    const bool comparison = comparison_owns_scoped_textures(&two_pixels);
    const bool player = player_releases_artwork(two_pixels);
    bool equivalent = true;
    for (int widget = 0; widget < 3; ++widget)
        for (int custom = 0; custom < 2; ++custom)
            for (int dark = 0; dark < 2; ++dark)
                equivalent = photo_pixels_match(widget, custom != 0, dark != 0) && equivalent;
    if (!passed) fprintf(stderr, "Media upload, reuse, replacement or release did not match source lifetime.\n");
    if (!wheel) fprintf(stderr, "The disabled viewer did not pass wheel input to its scroll area.\n");
    if (!crop) fprintf(stderr, "The initialized or resized crop did not preserve its aspect and minimum side.\n");
    if (!comparison) fprintf(stderr, "The comparison did not retain and release scoped image textures.\n");
    if (!player) fprintf(stderr, "The player did not retain and release artwork with the expected transport parts.\n");
    return passed && avatars_released && wheel && crop && comparison && player && equivalent ? 0 : 1;
}
