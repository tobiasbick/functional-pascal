# Managed CUI binding

The [binding reference](../../../src/Fpas.Gui.Cui/README.md) covers available
controls, test scope and platform verification. Application and SDK publishing
belong to [FPAS publishing](fpas-publishing.md).

## Native callbacks

- [ ] If native callbacks are introduced, root their state/delegates for their
  full lifetime, handle live-resize reentrancy and surface failures after leaving
  native frames. Managed exceptions must not cross the C ABI.

## Remaining acceptance

- [ ] Assert owned resource counts across repeated sessions, allocation failures
  and resource replacement. Disposal/state tests alone do not establish leak freedom.
- [ ] Exercise the finished binding in final NativeAOT consumers as part of the
  publishing work, including static linking and all newly added entry points.

The application-level API belongs to the [GUI host](gui-host.md); the low-level
binding and its fixtures do not define a second FPAS application model.
