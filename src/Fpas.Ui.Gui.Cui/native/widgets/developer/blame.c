#include "../../fpas_cui.h"

void fpas_cui_blame(cui_ctx *ui, const char *id, const char *const * lines, int lines_count, const int * line_commits, int line_commits_count, const cui_blame_commit * revisions, int revisions_count, float height, const char * language, int64_t now, int first_line, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    if (line_commits_count != lines_count) lines_count = 0;
    /* The caller's buffers can reuse addresses between frames. */
    const cui_id state_key = key ^ 0xB1A3u;
    fpas_own(ui, key, state_key, 0, 0);
    cui__blm_state *cache = cui__state_lazy(ui, state_key, sizeof *cache, false);
    if (cache) cache->lines = NULL;
    const cui_blame_opts o = { .width = fpas_extent(width_kind, width), .height = height, .lang = language ? cui__frame_strdup(ui, language, -1) : NULL, .now = now, .first_line = first_line };
    cui_push_disabled(ui, !enabled);
    cui_blame_result r = cui_blame_ex(ui,id,lines,line_commits,lines_count,revisions,revisions_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->action=r.clicked;
    out->hover=r.hovered;
}
