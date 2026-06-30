# Std.Tui views

The old retained public view API (`Application.HostRegisterView`, `Application.HostSetViewRect`, `Application.HostSetViewParent`, and related calls) is no longer registered.

`ViewId` still exists because transition frame and query APIs return or accept it. New Turbo Vision spike code should use `TuiDialog` and `TuiButton` handles instead of constructing `ViewId` values.

## See Also

- [Application](README.md)
- [Frame transition API](frames.md)
