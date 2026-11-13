/* CUI-035 (docs/future/cui-upstream-bugs.md#cui-035): the arc's painter
   uses the live clock and requests another frame even under reduced motion. */
static void fpas_dev_paint_still_spinner(cui_ctx *ui, cui_node *node, void *data) {
    const double time = ui->time;
    const bool animating = ui->animating;
    ui->time = 0;
    cui__paint_spinner(ui, node, data);
    ui->time = time;
    ui->animating = animating;
}

static void fpas_dev_reduce_spinners(cui_ctx *ui, int first) {
    if (ui->theme.motion > 0) return;
    for (int i = first; i < ui->node_count; ++i)
        if (ui->nodes[i].paint == cui__paint_spinner) ui->nodes[i].paint = fpas_dev_paint_still_spinner;
}

static fpas_dev_result fpas_dev_empty(void) {
    return (fpas_dev_result){ .index = -1, .action = -1, .hover = -1 };
}
