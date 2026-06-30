# Std.Tui views

The old retained public view API (`Application.HostRegisterView`, `Application.HostSetViewRect`, `Application.HostSetViewParent`, and related calls) is no longer registered or reachable from bytecode.

The retained view query APIs are also no longer public. Use the Turbo Vision facade (`Application.CreateDialog`, `Application.CreateButton`, `Application.AddChild`, and related calls) for new code.

The internal `ViewRegistry` still exists only for the transitional host loop; it cannot be populated from Pascal anymore.

## See Also

- [Application](README.md)
- [VM bridge](vm-bridge.md)
