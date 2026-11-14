#include "../../fpas_cui.h"

void fpas_cui_live_cursors(cui_ctx *ui, const char *id, const cui_live_user *people, int count,
    int transform, cui_node_view view, int no_glide) {
    const cui_live_cursors_opts options = { .view = transform ? &view : NULL, .no_glide = no_glide != 0 };
    cui_live_cursors_ex(ui, id, people, count, &options);
}
