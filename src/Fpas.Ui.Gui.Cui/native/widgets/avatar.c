#include "../fpas_cui.h"

int fpas_cui_avatar(cui_ctx *ui, const char *id, const char *name, const char *path,
    const char *status, float size, int square, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const fpas_avatar_image *image = fpas_avatar_image_for(ui, key, path);
    const cui_avatar_opts options = {
        .size = size, .square = square != 0, .status = status, .image = image->texture
    };
    *node = ui->node_count;
    cui_avatar_ex(ui, name, &options);
    ui->nodes[*node].id = key;
    ui->last_node = *node;
    ui->last_id = key;
    return image->texture != 0;
}
