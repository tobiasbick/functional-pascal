# Native CUI painting

Native canvas drawing, final layout geometry, Unicode text measurement, image
caching, hover/press proposals, user text scaling and reduced motion. The
system-aware theme supplies the same colors to widgets and custom drawing.

Run `fpas run examples/ui/gui/painting/Painting.fpasproj`. An optional trusted
TTF/OTF filename enables **Load font**; **Restore font** restores the built-in
body role. Every loaded face remains owned by the window until close. Pass a
new image pixel array only when its contents change; retained sources are immutable.

Paint runs after layout on the owner thread. Its drawing capability expires
when the callback returns. It cannot reenter the host or change application state.
