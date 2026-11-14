#include "../../fpas_cui.h"

typedef struct {
    cui_texture texture;
    char *path;
    int width, height;
    int64_t version;
} fpas_media_image;

static void fpas_media_release(cui_ctx *ui, void *data) {
    fpas_media_image *image = data;
    if (image->texture) cui_image_destroy(ui, image->texture);
    cui__free(ui, image->path);
}

static fpas_media_image *fpas_media_resolve(cui_ctx *ui, cui_id slot, const fpas_media_source *source) {
    fpas_media_image *image = cui_state(ui, slot, sizeof *image);
    cui__state_set_dtor(ui, slot, fpas_media_release);
    const bool same = source->path ? image->path && strcmp(image->path, source->path) == 0
        : !image->path && image->version == source->version
            && image->width == source->width && image->height == source->height;
    if (!same) {
        fpas_media_release(ui, image);
        *image = (fpas_media_image){ .version = source->version };
        if (source->path) {
            const size_t length = strlen(source->path) + 1;
            image->path = cui__alloc(ui, length);
            memcpy(image->path, source->path, length);
            image->texture = cui_image_load(ui, source->path, &image->width, &image->height);
        } else {
            image->width = source->width;
            image->height = source->height;
            image->texture = cui_image_create(ui, image->width, image->height, source->pixels);
        }
    }
    return image;
}
