// [Documentation](../../../../docs/ui/gui/app-chrome/window.md)
/* Window placement adapter with scalar arguments, so the managed side needs no struct layout. */

/** Reads the window's normal size and place and the state it opens in again.
 * @param app The window.
 * @param x Receives the left edge of the content in desktop coordinates.
 * @param y Receives the top edge of the content in desktop coordinates.
 * @param width Receives the normal logical width.
 * @param height Receives the normal logical height.
 * @param has_position Receives 1 when the system reports the position.
 * @return CUI_WINDOW_MAXIMIZED when the window is or comes back maximized, otherwise CUI_WINDOW_NORMAL. */
CUI_API int fpas_cui_placement(cui_app *app, int *x, int *y, int *width, int *height, int *has_position) {
    cui_window_placement placement;
    if (!cui_app_placement(app, &placement)) memset(&placement, 0, sizeof placement);
    *x = placement.x;
    *y = placement.y;
    *width = placement.width;
    *height = placement.height;
    *has_position = placement.has_position ? 1 : 0;
    return (int)placement.state;
}
