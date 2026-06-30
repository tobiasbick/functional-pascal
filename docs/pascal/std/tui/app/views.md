# Std.Tui views

The retained `ViewRegistry`, modal stack, and host widget runtime are removed. Pascal cannot register or query retained views anymore.

Use the Turbo Vision facade (`Application.CreateDialog`, `Application.CreateButton`, `Application.AddChild`, and related calls) for new UI code.

The hosted `Application.Run` loop still supports global handlers (`OnPaint`, `OnKeyPressed`, `OnResize`, and related `HostRegister*` calls) plus screen queries.

## See Also

- [Application](README.md)
- [VM bridge](vm-bridge.md)
